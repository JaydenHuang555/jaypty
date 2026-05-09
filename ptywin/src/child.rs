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
        GetProcessId, INFINITE, RegisterWaitForSingleObject, UnregisterWait, WT_EXECUTEDEFAULT,
        WT_EXECUTEINWAITTHREAD, WT_EXECUTEONLYONCE,
    },
};

pub mod watchdog;

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
}

extern "system" fn child_exit_callback(context: *mut c_void, timed_out: BOOLEAN) {
    if timed_out != 0 {
        return;
    }
    let event: Box<_> = unsafe { Box::from_raw(context as *mut ChildExitCallback) };
    event.sender.send(false).ok();
}

struct ChildExitCallback {
    sender: Sender<bool>,
}

pub struct ChildWatchDog {
    wait_handle: AtomicPtr<c_void>,
    child_handle: AtomicPtr<c_void>,
    pid: Option<NonZeroU32>,
    child_running: bool,
    reciever: Arc<Mutex<Receiver<bool>>>,
}

impl Drop for ChildWatchDog {
    fn drop(&mut self) {
        unsafe {
            UnregisterWait(self.wait_handle.load(Ordering::Relaxed) as HANDLE);
        }
    }
}

impl ChildWatchDog {
    pub fn new(child: HANDLE) -> Self {
        let (event_tx, event_rx) = std::sync::mpsc::channel();

        let mut wait_handle = ptr::null_mut();
        let exit_ref = Box::new(ChildExitCallback { sender: event_tx });
        unsafe {
            let stat = RegisterWaitForSingleObject(
                &mut wait_handle,
                child,
                Some(child_exit_callback),
                Box::into_raw(exit_ref).cast(),
                INFINITE,
                WT_EXECUTEINWAITTHREAD | WT_EXECUTEONLYONCE,
            );
            if stat == 0 {
                panic!("unable to create wait handle");
            }
        }
        let pid = unsafe { NonZeroU32::new(GetProcessId(child)) };
        Self {
            wait_handle: AtomicPtr::from(wait_handle),
            child_handle: AtomicPtr::from(child),
            pid,
            child_running: true,
            reciever: Arc::new(Mutex::new(event_rx)),
        }
    }

    pub fn update_child_running(&mut self) -> bool {
        if !self.child_running {
            return false;
        }
        for item in self.reciever.lock().unwrap().iter() {
            if !item {
                println!("child is not running");
                self.child_running = false;
                return false;
            }
        }
        true
    }

    pub fn child_handle(&self) -> HANDLE {
        self.child_handle.load(Ordering::Relaxed) as HANDLE
    }

    pub fn pid(&self) -> Option<NonZeroU32> {
        self.pid.clone()
    }
}
