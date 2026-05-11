use std::{
    ffi::c_void,
    io::{Read, Write},
    sync::{
        Arc,
        atomic::AtomicPtr,
        mpsc::{self, Receiver, Sender},
    },
    task::Wake,
};

use jaypty_core::{Options, Result, UnDefinedPseudoTerminalIO, io::PollingIntrestRegisterIO};
use jaypty_error::{OsEmptyResult, OsResult, SystemError};
use polling::{Event, Poller};
use windows_sys::Win32::System::{Console::COORD, Threading::TerminateProcess};

use super::ContpyHandle;
use crate::factory::ContpySpawn;
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

unsafe impl Send for ContpyPseudoTerminalIO {}

impl PollingIntrestRegisterIO<SystemError> for ContpyPseudoTerminalIO {
    unsafe fn register(
        &mut self,
        poller: &std::sync::Arc<polling::Poller>,
        intrest: Event,
        mode: Option<polling::PollMode>,
    ) -> OsEmptyResult {
        self.cin.register(poller, intrest, mode);
        self.cout.register(poller, intrest, mode);
        Ok(())
    }

    fn unregister(&mut self, _: &Arc<Poller>) -> OsEmptyResult {
        self.cin.unregister();
        self.cout.unregister();
        Ok(())
    }

    fn reregister(
        &mut self,
        poller: &std::sync::Arc<polling::Poller>,
        intrest: Event,
        mode: Option<polling::PollMode>,
    ) -> OsEmptyResult {
        unsafe { self.register(poller, intrest, mode) }
    }
}

impl UnDefinedPseudoTerminalIO<R, W, WinChildWatchdogIO, SystemError> for ContpyPseudoTerminalIO {
    fn new(options: jaypty_core::Options) -> OsResult<Self> {
        let mut spawn = ContpySpawn::spawn(options)?;
        let child_handle = factory::watch(&mut spawn)?;
        let cin = factory::cin(&mut spawn)?;
        let cout = factory::cout(&mut spawn)?;

        Ok(Self {
            cin,
            cout,
            handle: spawn.handle.take().expect("unable to take handle"),
            child_handle,
        })
    }

    fn resize(&mut self, size: jaypty_core::PtySize) -> OsResult<()> {
        unsafe {
            super::loaded_symbols().resize(
                self.handle,
                COORD {
                    X: size.columns as i16,
                    Y: size.rows as i16,
                },
            );
        }
        Ok(())
    }

    fn latch_watchdog(&self) -> OsResult<WinChildWatchdogIO> {
        WinChildWatchdogIO::latch(self.child_handle.load(std::sync::atomic::Ordering::Relaxed))
    }

    fn kill_child(&mut self) -> OsEmptyResult {
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
