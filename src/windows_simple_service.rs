//! Provides building blocks for "simple" services on Windows.
use std::ffi::OsString;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::time::Duration;
use windows_service::service::ServiceControl;
use windows_service::service::ServiceControlAccept;
use windows_service::service::ServiceExitCode;
use windows_service::service::ServiceState;
use windows_service::service::ServiceStatus;
use windows_service::service::ServiceType;
use windows_service::service_control_handler;
use windows_service::service_control_handler::ServiceControlHandlerResult;

use crate::windows_ffi_service_main::dispatch_main;
use crate::windows_ffi_service_main::set_service_main;
use crate::windows_ffi_service_main::FfiServiceMainFn;
use crate::ServiceMain;

pub type EventHandlerFactory<T> = dyn Fn(Sender<T>) -> Box<EventHandler>;
pub type EventHandler = dyn Fn(ServiceControl) -> ServiceControlHandlerResult + Send;

/// Generate a default event handler for Windows Service control events. Sends `()`
/// to `close_sender` when a stop control event is received.
pub fn default_event_handler(close_sender: Sender<()>) -> Box<EventHandler> {
    Box::new(
        move |control_event: ServiceControl| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                ServiceControl::Stop => {
                    close_sender.send(()).unwrap();
                    ServiceControlHandlerResult::NoError
                }
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        },
    )
}

/// Create an [`FfiServiceMainFn`] for a service with a given name and [`ServiceMain`] core function.
/// You should usually use [`run_simple_service`] instead.
pub fn make_simple_service_runner(
    service_name: String,
    service_main: Box<ServiceMain<()>>,
) -> Box<FfiServiceMainFn> {
    Box::new(move |start_params: Vec<OsString>| {
        let service_name = service_name.clone();
        let service_main = service_main.as_ref();
        let _ = move || -> anyhow::Result<()> {
            let (shutdown_tx, shutdown_rx) = channel();

            let event_handler = default_event_handler(shutdown_tx);

            let status_handle =
                service_control_handler::register(service_name.as_str(), event_handler)?;
            status_handle.set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })?;

            let start_params = start_params
                .iter()
                .map(|os| os.to_string_lossy().to_string())
                .collect();
            service_main(shutdown_rx, std::env::args().collect(), Some(start_params))?;

            status_handle.set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })?;

            Ok(())
        }();
    })
}

/// Run a ServiceMain function as a simple Windows Service. "Simple" in this case
/// means it only responds to the ServiceControl::Stop event and is run as a
/// standalone process.
pub fn run_simple_service(
    service_name: String,
    service_main: Box<ServiceMain<()>>,
) -> anyhow::Result<()> {
    set_service_main(make_simple_service_runner(
        service_name.clone(),
        service_main,
    ))
    .expect("failed to set service main");
    dispatch_main(service_name);
    Ok(())
}
