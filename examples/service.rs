use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use universal_service::universal_service_main;

fn main() -> anyhow::Result<()> {
    universal_service_main(
        "test_service".to_owned(),
        Box::new(
            |shutdown_rx: std::sync::mpsc::Receiver<()>,
             _binary_args: Vec<String>,
             _start_parameters: Option<Vec<String>>|
             -> anyhow::Result<()> {
                #[cfg(windows)]
                let path = "C:/Windows/Temp/test.txt";
                #[cfg(not(windows))]
                let path = "/tmp/test.txt";
                let mut fh = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)?;
                write!(
                    fh,
                    "example service ran at time: {}",
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                )?;
                loop {
                    match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
                        Ok(_) | Err(RecvTimeoutError::Disconnected) => break,
                        Err(RecvTimeoutError::Timeout) => (),
                    };
                }
                Ok(())
            },
        ),
    )?;
    Ok(())
}
