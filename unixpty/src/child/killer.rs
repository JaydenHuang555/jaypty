use std::{
    process::Child,
    task::Wake,
    thread::{self, JoinHandle, Thread},
};

use jaypty_core::{
    ErrorFactory, FactoriedErrorKind, Result, SystemError,
    child::{ChildStatus, killer::ConsumedChildKiller},
};

pub struct ConsumedPosixChildKiller(pub(crate) Child);

impl ConsumedPosixChildKiller {
    pub(crate) fn consume(self) -> Child {
        self.0
    }
}

impl ConsumedChildKiller for ConsumedPosixChildKiller {
    fn blocking(self) -> Result<ChildStatus> {
        let mut child = self.consume();
        let _ = child.kill().map_err(|e| {
            ErrorFactory::kind(FactoriedErrorKind::ChildKillFailed)
                .with_internal(SystemError::KillChildFailure(e))
        })?;
        child
            .wait()
            .map_err(|e| {
                ErrorFactory::kind(FactoriedErrorKind::ChildKillFailed)
                    .with_internal(SystemError::WaitChildRelease(e))
            })
            .map(|stat| ChildStatus::Dead(stat.code().unwrap_or(0)))
    }

    fn nonblocking(self) -> JoinHandle<Result<ChildStatus>> {
        thread::spawn(move || self.blocking())
    }
}
