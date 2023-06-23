//! Low-level library that encapsulates the FFI interfaces from the `windows-service` crate
//! in a way that does not require every service binary to call `define_windows_service!`.

use core::fmt;
use std::ffi::OsString;

use once_cell::sync::OnceCell;
use windows_service::define_windows_service;
use windows_service::service_dispatcher;

/// The signature for the service `main()` function or closure that can be
/// injected into the FFI service main entrypoint function via `set_service_main`.
pub type FfiServiceMainFn = dyn Fn(Vec<OsString>) + Sync + Send;

static SERVICE_MAIN: OnceCell<Box<FfiServiceMainFn>> = OnceCell::new();

/// Custom error type for signaling that injection of the global `SERVICE_MAIN`
/// `OnceCell` failed.
#[derive(Debug, Clone, PartialEq)]
pub enum FFIServiceMainError {
    FailedToSetGlobalServiceMain,
}

impl fmt::Display for FFIServiceMainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

/// Inject a [`FfiServiceMainFn`] as the entrypoint for Windows services in this process.
/// This injection can only be called once per process.
///
/// ```rust
/// use windows_service_wrapper::ffi_service_main2::set_service_main;
/// set_service_main(Box::new(move |_| { println!("do nothing")}));
/// ```
pub fn set_service_main(f: Box<FfiServiceMainFn>) -> Result<(), FFIServiceMainError> {
    SERVICE_MAIN
        .set(f)
        .or(Err(FFIServiceMainError::FailedToSetGlobalServiceMain))
}

fn service_main(args: Vec<OsString>) {
    let main = SERVICE_MAIN.get().expect("SERVICE_MAIN not initialized");
    main(args);
}

// creates the extern "system" fn {...}
define_windows_service!(ffi_service_main, service_main);

/// Start the injected service main function into the Windows Service manager. You can use this
/// dispatch function directly if the core service functions are not suitable.
pub fn dispatch_main(service_name: String) {
    let _ = service_dispatcher::start(service_name, ffi_service_main);
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::sync::Arc;
    use std::sync::Mutex;

    use super::service_main;
    use super::SERVICE_MAIN;

    #[test]
    fn test_service_main_fn() {
        let count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        let inner_count = count.clone();
        let test_main = Box::new(move |_: Vec<OsString>| {
            let mut count = inner_count.lock().expect("failed to lock");
            *count += 1;
        });
        match SERVICE_MAIN.set(test_main) {
            Ok(_) => (),
            Err(_) => panic!("failed to set test_main"),
        }
        service_main(vec![]);
        assert_eq!(*count.lock().expect("inner"), 1);
    }
}
