use std::{
    num::NonZeroU32,
    os::raw::c_void,
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicPtr, Ordering},
        mpsc::{Receiver, Sender},
    },
};

use jaysync::mpsc;
use polling::{Poller, os::iocp::PollerIocpExt};
use windows_sys::Win32::{
    Foundation::{BOOLEAN, HANDLE},
    System::Threading::{
        GetExitCodeProcess, GetProcessId, INFINITE, RegisterWaitForSingleObject, UnregisterWait,
        WT_EXECUTEDEFAULT, WT_EXECUTEINWAITTHREAD, WT_EXECUTEONLYONCE,
    },
};

use crate::child::ChildProcess;

extern "system" fn child_exit_callback(context: *mut c_void, timed_out: BOOLEAN) {
    if timed_out != 0 {
        return;
    }
    log::info!("child callback");
    let event: Box<_> = unsafe { Box::from_raw(context as *mut ChildExitCallback) };
    let child = event.handle.load(Ordering::Relaxed) as HANDLE;
    let mut exit_code = 0_u32;

    let _ = unsafe { GetExitCodeProcess(child, &mut exit_code) };
    event.sender.send(exit_code).ok();
}

struct ChildExitCallback {
    sender: Sender<u32>,
    handle: AtomicPtr<c_void>,
}

pub struct ChildWatchDog {
    wait_handle: AtomicPtr<c_void>,
    exit_code: Option<u32>,
    alive: bool,
    reciever: Receiver<u32>,
}

impl Drop for ChildWatchDog {
    fn drop(&mut self) {
        unsafe {
            UnregisterWait(self.wait_handle.load(Ordering::Relaxed) as HANDLE);
        }
    }
}

impl ChildWatchDog {
    pub fn new(process_child: &ChildProcess) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let child = process_child.child_handle();

        let poller = Arc::new(Poller::new().unwrap());
        let exit_ref_poller = Arc::clone(&poller);

        let mut wait_handle = ptr::null_mut();
        let exit_ref = Box::new(ChildExitCallback {
            sender: tx,
            handle: AtomicPtr::from(process_child.child_handle()),
        });
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
        Self {
            wait_handle: AtomicPtr::from(wait_handle),
            exit_code: None,
            reciever: rx,
            alive: true,
        }
    }

    pub fn wait(&mut self) {
        if !self.alive {
            return;
        }
        let exit_code = self.reciever.recv().unwrap();
        self.exit_code = Some(exit_code);
    }

    pub fn is_alive(&self) -> bool {
        self.alive
    }
}
