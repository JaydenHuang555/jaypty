use std::{
    ffi::c_void,
    io,
    sync::atomic::{AtomicPtr, Ordering},
    thread,
};

use jaypty_core::{
    ErrorFactory, FactoriedError, FactoriedErrorKind, SystemError,
    child::{ChildStatus, consume::ConsumedChildConsumer, killer::ConsumedChildKiller},
};
use windows_sys::Win32::{
    Foundation::WAIT_FAILED,
    System::Threading::{GetExitCodeProcess, INFINITE, TerminateProcess, WaitForSingleObject},
};

use crate::child::{ChildHandle, WinChildWatchdogIO};

pub struct ConsumedContpyChildKiller(pub(crate) ChildHandle);

impl ConsumedContpyChildKiller {
    pub(crate) fn consume(self) -> ChildHandle {
        self.0
    }
}

impl ConsumedChildKiller for ConsumedContpyChildKiller {
    fn blocking(self) -> jaypty_core::Result<jaypty_core::child::ChildStatus> {
        let handle = self.consume();
        let _ = unsafe { TerminateProcess(handle.load(std::sync::atomic::Ordering::Relaxed), 1) };
        let wait_stat = unsafe { WaitForSingleObject(handle.load(Ordering::Relaxed), INFINITE) };
        if wait_stat == WAIT_FAILED {
            return Err(ErrorFactory::kind(FactoriedErrorKind::ChildKillFailed)
                .with_internal(SystemError::ChildFailedRegisterWait(wait_stat)));
        }
        let mut exit_code = 0u32;
        let stat = unsafe {
            GetExitCodeProcess(handle.load(Ordering::Relaxed), &mut exit_code as *mut u32)
        };
        if stat == 0 {
            return Err(
                ErrorFactory::kind(FactoriedErrorKind::ChildKillFailed).with_internal(
                    SystemError::ChildFailedToGetExitCode(io::Error::last_os_error()),
                ),
            );
        }
        Ok(ChildStatus::Dead(exit_code as i32))
    }

    fn nonblocking(
        self,
    ) -> std::thread::JoinHandle<jaypty_core::Result<jaypty_core::child::ChildStatus>> {
        thread::spawn(move || self.blocking())
    }
}
