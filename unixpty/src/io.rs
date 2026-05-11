use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    os::{fd::AsRawFd, unix::net::UnixStream},
    process::{Child, ExitStatus},
    sync::{Arc, Mutex},
};

use jaypty_core::{
    Options, OsEmptyResult, OsResult, SystemError, UnDefinedPseudoTerminalIO,
    io::PollingIntrestRegisterIO,
};
use libc::{F_GETFL, F_SETFL, FILE, O_NONBLOCK, SIGCHLD, SIGHUP, fcntl, winsize};
use polling::Poller;
use rustix::{fs::openat, io};
use signal_hook::{
    SigId,
    flag::register,
    low_level::{pipe, unregister},
};

use crate::{
    Error, Pty,
    factory::{self, Account},
    watchdog::SignalWatchDogIO,
};

pub struct UnixPseudoTerminalIO {
    child: Child,
    io: File,
}

impl Drop for UnixPseudoTerminalIO {
    fn drop(&mut self) {
        unsafe {
            libc::kill(self.child.id() as i32, SIGHUP);
        }
        // On some systems, calling wait or similar is necessary for the OS to release resources.
        // A process that terminated but has not been waited on is still around as a "zombie".
        // Leaving too many zombies around may exhaust global resources (for example process IDs).
        self.child.wait().expect("failed to wait for child to exit");
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
                .map_err(|e| SystemError::PollingPtyIntrestFailure(e))
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
            .map_err(|e| SystemError::PollingPtyIntrestFailure(e))
    }

    fn unregister(&mut self, poller: &Arc<Poller>) -> OsEmptyResult {
        poller
            .delete(&self.io)
            .map_err(|e| SystemError::PollingPtyIntrestFailure(e))
    }
}

impl UnDefinedPseudoTerminalIO<File, File, SignalWatchDogIO, SystemError> for UnixPseudoTerminalIO {
    fn new(options: Options) -> OsResult<Self> {
        let mut pty = Pty::spawn()
            .map_err(|errno| Error::CreatePty(errno))
            .unwrap();

        let mut cmd = factory::build_cmd(&options, &pty).unwrap();

        match cmd.spawn() {
            Ok(child) => {
                pty.set_master_nonblocking();

                Ok(Self {
                    child,
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

    fn latch_watchdog(&self) -> SignalWatchDogIO {
        SignalWatchDogIO::spawn(SIGHUP).unwrap()
    }

    fn kill_child(&mut self) -> jaypty_core::Result<()> {
        self.child.kill().map_err(|e| {
            jaypty_core::error::Error::new(ChildError::UnableToGetExitCode.into(), None)
        })
    }

    fn cin(&mut self) -> &mut File {
        &mut self.io
    }

    fn cout(&mut self) -> &mut File {
        &mut self.io
    }
}

impl Default for UnixPseudoTerminalIO {
    fn default() -> Self {
        Self::new(Options::default())
    }
}
