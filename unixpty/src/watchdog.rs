use std::{
    ffi::c_int,
    io::{ErrorKind, Read},
    os::unix::net::UnixStream,
    sync::mpsc::Receiver,
    time::Duration,
};

use jaypty_core::{
    OsEmptyResult, OsError, OsResult, SystemError,
    child::{ChildPollRegisterIO, ChildStatus, ChildWatchDogIO},
};
use libc::SIGCHLD;
use polling::{PollMode, Poller};
use signal_hook::{
    SigId,
    low_level::{pipe, unregister},
};

pub struct SignalWatchDogIO {
    listener: UnixStream,
    sig_id: SigId,
    status: ChildStatus,
}

impl Drop for SignalWatchDogIO {
    fn drop(&mut self) {
        unregister(self.sig_id);
    }
}

impl SignalWatchDogIO {
    pub fn spawn(signal: c_int) -> OsResult<Self> {
        let (pipe, listener) =
            UnixStream::pair().map_err(SystemError::SpawnChildExitListenerStreamFailure)?;
        let sig_id =
            pipe::register(signal, pipe).map_err(SystemError::SpawnChildPipeCallBackFailure)?;
        listener
            .set_nonblocking(true)
            .map_err(SystemError::SpawnChildExitListenerStreamFailure)?;
        Ok(Self {
            listener,
            sig_id,
            status: ChildStatus::Alive,
        })
    }
}

impl ChildPollRegisterIO<OsError> for SignalWatchDogIO {
    unsafe fn register(
        &mut self,
        poller: &std::sync::Arc<Poller>,
        intrest: polling::Event,
    ) -> OsEmptyResult {
        unsafe {
            poller
                .add(&self.listener, intrest)
                .map_err(SystemError::PollingChildIntrestFailure)
        }
    }

    unsafe fn reregister(
        &mut self,
        poller: &std::sync::Arc<Poller>,
        intrest: polling::Event,
    ) -> OsEmptyResult {
        poller
            .modify(&self.listener, intrest)
            .map_err(SystemError::PollingChildIntrestFailure)
    }

    unsafe fn unregister(&mut self, poller: &std::sync::Arc<Poller>) -> OsEmptyResult {
        poller
            .delete(&self.listener)
            .map_err(SystemError::PollingChildIntrestFailure)
    }
}

impl ChildWatchDogIO<SystemError> for SignalWatchDogIO {
    /// TODO: add support for getting the exit code from the child
    /// if it does exit
    fn status(&mut self) -> OsResult<ChildStatus> {
        if self.status.is_dead() {
            return Ok(self.status);
        } else {
            let mut buff = [0u8; 1];
            match self.listener.read(&mut buff) {
                Ok(n) => {
                    if n > 0 {
                        self.status = ChildStatus::Dead(0);
                    }
                }
                Err(_) => {}
            }
            Ok(self.status)
        }
    }

    fn is_dead(&self) -> OsResult<ChildStatus> {
        Ok(self.status)
    }

    fn wait(&mut self) -> OsResult<ChildStatus> {
        if self.status.is_dead() {
            return Ok(self.status);
        }

        let last_status = self.status;
        loop {
            std::thread::sleep(Duration::from_millis(200));
            if self.status()? != last_status {
                if self.status.is_dead() {
                    break;
                }
            }
        }
        return Ok(self.status);
    }
}
