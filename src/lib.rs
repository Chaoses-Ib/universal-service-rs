//! `universal-service` is a Rust crate that provides utilities for building services that
//! can run across multiple platforms.
//!
//! For Windows users, this library provides support for running services under the Windows
//! Service manager directly, rather than requiring Scheduled Tasks hacks or NSSM.

#[cfg(windows)]
pub mod windows_ffi_service_main;
#[cfg(windows)]
pub mod windows_simple_service;

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;

/// Type signature of a service's main function used by this library.
///
/// The three parameters are used as follows:
/// * `Receiver<T>`: a [`std::sync::mpsc::Receiver<T>`] that is used to transmit service stop
///   signals to the main function for handling.
/// * `Vec<String>`: the operating system arguments passed to the binary at process load time.
///   For a binary running as a Windows Service, these are the arguments passed to `-BinaryPath`
/// * `Option<Vec<String>>`: the start parameters passed as part of Windows Service startup, if
///   the binary is running as a Windows Service. This argument should be `None` if this is not
///   running as a Windows Service
pub type ServiceMain<T> =
    dyn Fn(Receiver<T>, Vec<String>, Option<Vec<String>>) -> anyhow::Result<T> + Sync + Send;

#[cfg(windows)]
use windows_service_detector::is_running_as_windows_service;
#[cfg(windows)]
use windows_simple_service::run_simple_service;

#[cfg(windows)]
/// Run a "universal" service main function. On Windows, this will do service environment detection
/// and choose whether to initialize as a Windows Service or a normal CLI binary. On other platforms
/// it will run as normal process (or "simple" in systemd parlance).
pub fn universal_service_main(
    service_name: String,
    service_main: Box<ServiceMain<()>>,
) -> anyhow::Result<()> {
    if is_running_as_windows_service().expect("failed to detect windows service environment") {
        run_simple_service(service_name, service_main)?;
    } else {
        run_simple_nonservice(service_name, service_main)?;
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn universal_service_main(
    service_name: String,
    service_main: Box<ServiceMain<()>>,
) -> anyhow::Result<()> {
    run_simple_nonservice(service_name, service_main)?;
    Ok(())
}

/// Run a ServiceMain function as a simple foreground process. Instead of registering
/// as a service, this runs the function with just a signal handler that sends to
/// the shutdown receiver.
pub fn run_simple_nonservice(
    _service_name: String,
    service_main: Box<ServiceMain<()>>,
) -> anyhow::Result<()> {
    let (shutdown_tx, shutdown_rx) = channel();

    ctrlc::set_handler(move || {
        let _ = shutdown_tx.send(());
    })
    .expect("failed to set signal handler");

    service_main(shutdown_rx, std::env::args().collect(), None)?;
    Ok(())
}
