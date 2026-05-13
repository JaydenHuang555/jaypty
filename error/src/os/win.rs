use std::{io, sync::PoisonError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContpyIOError {
    #[error("Failed to spawn io pipes when creating io for the contpy instance")]
    FailedSpawningIOPipes(io::Error),
    #[error("failed to create pty handle with return {0}")]
    FailedCreatingPtyHandle(i32),
    #[error("failed to create process with return {0}")]
    CreateProcessFailed(i32),
    #[error("unable to find contpy.dll")]
    UnableToFindContpyDll,
    #[error("Failed to load symbol {0} from contpy.dll")]
    UnableToLoadContpyDllSymbol(&'static str),
    #[error("unable to take child handle from spawn")]
    UnableToTakeChildHandleFromSpawn,
    #[error("unable to take io handle from spawn")]
    UnableToTakeIOFromSpawn,
    #[error("unable to aquire child watch dog lock")]
    UnableToAquireChildWatchDogLock,
    #[error("child watchdog recv failure for an mspc channel")]
    ChildWatchDogChannelDisconnected,
    #[error("child watch dog register intrest lock is poisoned")]
    ChildWatchDogRegisterMutexPoison,
    #[error("failed to register child wait with exit code {0}")]
    ChildFailedRegisterWait(u32),
    #[error("failed to get child's exit code with error {0}")]
    ChildFailedToGetExitCode(io::Error),
}

pub use ContpyIOError as SystemError;
