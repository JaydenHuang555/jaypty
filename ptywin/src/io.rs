use std::{
    ffi::c_void,
    io::{Read, Write},
    sync::{
        atomic::AtomicPtr,
        mpsc::{self, Receiver, Sender},
    },
    task::Wake,
};

use jaypty_core::{Options, PseudoTerminalIO, io::UnsafePseudoTerminalRegisterIO, tokens::Token};
use jwinpipe::polling::{PollingWakingNonBlockingPipeReader, PollingWakingNonBlockingPipeWriter};
use miow::pipe::AnonRead;
use polling::Event;
use windows_sys::Win32::System::{Console::COORD, Threading::TerminateProcess};

use super::ContpyHandle;
use crate::factory::ContpySpawn;
use crate::{RegisteredPoll, poll::Polled};
use crate::{child::WinChildWatchdogIO, factory};

type R = factory::io::R;
type W = factory::io::W;

pub struct ContpyPseudoTerminalIO {
    cout: R,
    cin: W,
    handle: ContpyHandle,
    child_handle: AtomicPtr<c_void>,
}

impl Drop for ContpyPseudoTerminalIO {
    fn drop(&mut self) {
        unsafe {
            super::loaded_symbols().close(self.handle);
        }
    }
}

impl Default for ContpyPseudoTerminalIO {
    fn default() -> Self {
        let options = Options::default();
        ContpyPseudoTerminalIO::new(options)
    }
}

unsafe impl Send for ContpyPseudoTerminalIO {}

impl UnsafePseudoTerminalRegisterIO for ContpyPseudoTerminalIO {
    unsafe fn register(
        &mut self,
        poller: &std::sync::Arc<polling::Poller>,
        intrest: Event,
        mode: Option<polling::PollMode>,
    ) {
        self.cin
            .register(poller, Token::CinWrite.keyify(intrest), mode);
        self.cout
            .register(poller, Token::CoutRead.keyify(intrest), mode);
    }

    unsafe fn unregister(&mut self) {
        self.cin.unregister();
        self.cout.unregister();
    }

    unsafe fn reregister(
        &mut self,
        poller: &std::sync::Arc<polling::Poller>,
        intrest: Event,
        mode: Option<polling::PollMode>,
    ) {
        unsafe {
            self.register(poller, intrest, mode);
        }
    }
}

impl PseudoTerminalIO<R, W, WinChildWatchdogIO> for ContpyPseudoTerminalIO {
    fn new(options: jaypty_core::Options) -> Self {
        let mut spawn = ContpySpawn::spawn(options);
        let child_handle = factory::watch(&mut spawn);
        let cin = factory::cin(&mut spawn);
        let cout = factory::cout(&mut spawn);

        Self {
            cin,
            cout,
            handle: spawn.handle.take().expect("unable to take handle"),
            child_handle,
        }
    }

    fn resize(&mut self, size: jaypty_core::PtySize) {
        unsafe {
            super::loaded_symbols().resize(
                self.handle,
                COORD {
                    X: size.columns as i16,
                    Y: size.rows as i16,
                },
            );
        }
    }

    fn spawn_and_latch_child_watchdog(&self) -> WinChildWatchdogIO {
        WinChildWatchdogIO::latch(self.child_handle.load(std::sync::atomic::Ordering::Relaxed))
    }

    fn kill_child(&mut self) -> jaypty_core::Result<()> {
        let _ = unsafe {
            TerminateProcess(
                self.child_handle.load(std::sync::atomic::Ordering::Relaxed),
                1,
            )
        };
        Ok(())
    }

    fn cin(&mut self) -> &mut W {
        &mut self.cin
    }

    fn cout(&mut self) -> &mut R {
        &mut self.cout
    }
}

impl Write for ContpyPseudoTerminalIO {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.cin.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.cin.flush()
    }
}

impl Read for ContpyPseudoTerminalIO {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cout.read(buf)
    }
}
