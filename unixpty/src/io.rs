use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    os::{fd::AsRawFd, unix::net::UnixStream},
    process::{Child, ExitStatus},
    sync::{Arc, Mutex},
};

use jaypty_core::{Options, PseudoTerminalIO, error::ChildError, io::PollingIntrestRegisterIO};
use libc::{F_GETFL, F_SETFL, FILE, O_NONBLOCK, SIGCHLD, SIGHUP, fcntl};
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
    file: File,
}

impl UnixPseudoTerminalIO {
    pub fn new(options: Options) -> crate::Result<Self> {
        let mut pty = Pty::spawn().map_err(|errno| Error::CreatePty(errno))?;

        let mut cmd = factory::build_cmd(&options, &pty)?;

        match cmd.spawn() {
            Ok(child) => {
                pty.set_master_nonblocking();

                Ok(Self {
                    child,
                    file: File::from(pty.master),
                })
            }
            Err(e) => Err(Error::IOError(e)),
        }
    }
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
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl Read for UnixPseudoTerminalIO {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl PollingIntrestRegisterIO for UnixPseudoTerminalIO {
    unsafe fn register(
        &mut self,
        poller: &Arc<polling::Poller>,
        intrest: polling::Event,
        mode: Option<polling::PollMode>,
    ) {
        unsafe {
            poller
                .add_with_mode(
                    &self.file,
                    intrest,
                    mode.unwrap_or(polling::PollMode::Oneshot),
                )
                .ok();
        }
    }

    fn reregister(
        &mut self,
        poller: &Arc<polling::Poller>,
        intrest: polling::Event,
        mode: Option<polling::PollMode>,
    ) {
        mode.as_ref()
            .map_or_else(
                || poller.modify(&self.file, intrest),
                |m| poller.modify_with_mode(&self.file, intrest, *m),
            )
            .ok();
    }

    fn unregister(&mut self, poller: &Arc<Poller>) {
        poller.delete(&self.file).ok();
    }
}

impl PseudoTerminalIO<File, File, SignalWatchDogIO> for UnixPseudoTerminalIO {
    fn new(_options: Options) -> Self {
        todo!()
    }

    fn resize(&mut self, _size: jaypty_core::PtySize) {
        todo!()
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
        &mut self.file
    }

    fn cout(&mut self) -> &mut File {
        &mut self.file
    }
}

impl Default for UnixPseudoTerminalIO {
    fn default() -> Self {
        Self::new(Options::default()).unwrap()
    }
}
