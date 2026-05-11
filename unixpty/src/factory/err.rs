use rustix::io::Errno;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum FactoryError {
    #[error("failed to open pty. errno is {0}")]
    FailedOpenPty(Errno),
    #[error("failed to grant perms to the master pty handle. errno is: {0}")]
    GrantPerms(Errno),
}

pub type Result<T> = std::result::Result<T, FactoryError>;
