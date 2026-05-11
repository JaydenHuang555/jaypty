use std::io;

use rustix::io::Errno;
use thiserror::Error;

use crate::factory::account::AccountGetterError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unable to create pty. errno is: {0}")]
    CreatePty(Errno),
    #[error("failed to get account: {0}")]
    ShellGetter(#[from] AccountGetterError),
    #[error("faild to clone with error: {0}")]
    UnableToCloneHandle(std::io::Error),
    #[error("failed to spawn unix stream with error {0}")]
    FailedSpawningUnixStream(io::Error),
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
    #[error("io error exit code: {0}")]
    IOErrorNo(i32),
}

pub type Result<T> = std::result::Result<T, Error>;
