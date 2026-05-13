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
    #[error("failed to spawn listener for SIGCHLD with error {0}")]
    SpawnChildExitListenerStreamFailure(io::Error),
    #[error("failed to spawn listener for SIGCHLD with error {0}")]
    SpawnChildPipeCallBackFailure(io::Error),
    #[error("failed to kill child process with error {0}")]
    KillChildFailure(io::Error),
    #[error("failed to wait for the child process to release resources with error {0}")]
    WaitChildRelease(io::Error),
}

pub use UnixPtyIOError as SystemError;
