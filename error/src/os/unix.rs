use std::io;

use thiserror::Error;
#[derive(Debug, Error)]
pub enum UnixPtyIOError {
    #[error("failed to attach polling intrest to pty io with error {0}")]
    PollingPtyIntrestFailure(io::Error),
    #[error("failed to attach polling intrest to child watchdog with error {0}")]
    PollingChildIntrestFailure(io::Error),
    #[error("failed to spawn the cmd process with error {0}")]
    FailedSpawnProcess(io::Error),
}

pub use UnixPtyIOError as SystemError;
