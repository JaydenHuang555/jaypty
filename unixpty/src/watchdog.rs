use std::{
    ffi::c_int,
    io::{ErrorKind, Read},
    os::unix::net::UnixStream,
    sync::mpsc::Receiver,
};

use jaypty_core::{
    OsEmptyResult, OsError, OsResult, SystemError,
    child::{ChildPollRegisterIO, ChildWatchDogIO},
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
    detected_signal: bool,
}

impl Drop for SignalWatchDogIO {
    fn drop(&mut self) {
        unregister(self.sig_id);
    }
}

impl SignalWatchDogIO {
    pub fn spawn(signal: c_int) -> super::Result<Self> {
        let (pipe, listener) = UnixStream::pair()?;
        let sig_id = pipe::register(signal, pipe)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            sig_id,
            detected_signal: false,
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
    fn status(&mut self) -> Option<OsResult> {
        if self.detected_signal {
            Some(Ok(0))
        } else {
            let mut buff = [0u8; 1];
            match self.listener.read(&mut buff) {
                Ok(n) => {
                    if n > 0 {
                        self.detected_signal = true;
                        return Some(Ok(0));
                    }
                }
                Err(e) => {
                    return None;
                }
            }

            None
        }
    }

    fn is_dead(&self) -> bool {
        self.detected_signal
    }

    fn wait(&mut self) {
        self.listener.set_nonblocking(false).unwrap();
        let mut buff = [0u8; 1];
        match self.listener.read(&mut buff) {
            Ok(e) => {
                self.listener.set_nonblocking(true).ok();
                return;
            }
            Err(_) => {
                self.listener.set_nonblocking(true).ok();
                return;
            }
        }
    }
}
