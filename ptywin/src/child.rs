use std::{
    ffi::c_void,
    num::NonZeroU32,
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicPtr, Ordering},
        mpsc::{Receiver, Sender},
    },
};

use polling::{Event, PollMode, Poller};
use windows_sys::Win32::{
    Foundation::{BOOLEAN, HANDLE},
    System::Threading::{
        ExitProcess, GetProcessId, INFINITE, RegisterWaitForSingleObject, TerminateProcess,
        UnregisterWait, WT_EXECUTEDEFAULT, WT_EXECUTEINWAITTHREAD, WT_EXECUTEONLYONCE,
    },
};

pub struct ChildProcess {
    pid: Option<NonZeroU32>,
    handle: AtomicPtr<c_void>,
}

impl ChildProcess {
    pub fn new(handle: HANDLE) -> Self {
        let pid = unsafe { NonZeroU32::new(GetProcessId(handle)) };
        Self {
            pid,
            handle: AtomicPtr::from(handle),
        }
    }

    pub fn child_handle(&self) -> *mut c_void {
        self.handle.load(Ordering::Relaxed)
    }

    pub fn kill(&self) {
        unsafe {
            TerminateProcess(self.handle.load(Ordering::Relaxed) as *mut c_void, 0);
        }
    }
}
