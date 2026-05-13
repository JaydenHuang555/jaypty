use crate::Cin;
use crate::Cout;
use crate::child::ConsumedPosixChildConsumer;
use crate::child::killer::ConsumedPosixChildKiller;
use std::{
    fs::File,
    io::{Read, Write},
    os::{fd::AsRawFd, unix::net::UnixStream},
    process::{Child, ExitStatus},
    sync::{Arc, Mutex},
};

use jaypty_core::child::consume::ConsumedChildConsumer;
use jaypty_core::{
    Options, OsEmptyResult, OsResult, SystemError, UnDefinedPseudoTerminalIO,
    io::PollingIntrestRegisterIO,
};
use libc::{F_GETFL, F_SETFL, FILE, O_NONBLOCK, SIGCHLD, SIGHUP, fcntl, winsize};
use polling::Poller;
use rustix::{fs::openat, io};

use crate::{
    Error, Pty,
    child::watchdog::SignalWatchDogIO,
    factory::{self, Account},
};

pub struct UnixPseudoTerminalIO {
    child: Option<Child>,
    io: File,
}

impl Drop for UnixPseudoTerminalIO {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = unsafe { libc::kill(child.id() as i32, SIGHUP) };
            let _ = child.wait();
        }
    }
}

impl Write for UnixPseudoTerminalIO {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.io.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.io.flush()
    }
}

impl Read for UnixPseudoTerminalIO {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.io.read(buf)
    }
}

impl PollingIntrestRegisterIO<SystemError> for UnixPseudoTerminalIO {
    unsafe fn register(
        &mut self,
        poller: &Arc<polling::Poller>,
        intrest: polling::Event,
        mode: Option<polling::PollMode>,
    ) -> OsEmptyResult {
        unsafe {
            poller
                .add_with_mode(
                    &self.io,
                    intrest,
                    mode.unwrap_or(polling::PollMode::Oneshot),
                )
                .map_err(SystemError::PollingPtyIntrestFailure)
        }
    }

    fn reregister(
        &mut self,
        poller: &Arc<polling::Poller>,
        intrest: polling::Event,
        mode: Option<polling::PollMode>,
    ) -> OsEmptyResult {
        mode.as_ref()
            .map_or_else(
                || poller.modify(&self.io, intrest),
                |m| poller.modify_with_mode(&self.io, intrest, *m),
            )
            .map_err(SystemError::PollingPtyIntrestFailure)
    }

    fn unregister(&mut self, poller: &Arc<Poller>) -> OsEmptyResult {
        poller
            .delete(&self.io)
            .map_err(SystemError::PollingPtyIntrestFailure)
    }
}

impl
    UnDefinedPseudoTerminalIO<
        Cout,
        Cin,
        SignalWatchDogIO,
        SystemError,
        ConsumedPosixChildKiller,
        ConsumedPosixChildConsumer,
    > for UnixPseudoTerminalIO
{
    fn new(options: Options) -> OsResult<Self> {
        let mut pty = Pty::spawn().map_err(Error::CreatePty).unwrap();

        let mut cmd = factory::build_cmd(&options, &pty).unwrap();

        match cmd.spawn() {
            Ok(child) => {
                pty.set_master_nonblocking();

                Ok(Self {
                    child: Some(child),
                    io: File::from(pty.master),
                })
            }
            Err(e) => Err(SystemError::FailedSpawnProcess(e)),
        }
    }

    fn resize(&mut self, size: jaypty_core::PtySize) -> OsEmptyResult {
        let win = winsize {
            ws_row: size.rows as u16,
            ws_col: size.columns as u16,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe { libc::ioctl(self.io.as_raw_fd(), libc::TIOCSWINSZ, &win as *const _) };
        Ok(())
    }

    fn latch_watchdog(&self) -> OsResult<SignalWatchDogIO> {
        SignalWatchDogIO::spawn()
    }

    fn kill_child(&mut self) -> OsEmptyResult {
        if let Some(mut child) = self.child.take() {
            let _ = unsafe { libc::kill(child.id() as i32, SIGHUP) };
            let _ = child.wait();
        }
        Ok(())
    }

    fn cin(&mut self) -> &mut Cin {
        &mut self.io
    }

    fn cout(&mut self) -> &mut Cout {
        &mut self.io
    }

    fn consume_child(&mut self) -> Option<ConsumedPosixChildConsumer> {
        self.child.take().map(|c| ConsumedPosixChildConsumer(c))
    }
}
