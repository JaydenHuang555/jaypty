use std::{
    ffi::c_void,
    future::Pending,
    num::NonZeroU32,
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicPtr, Ordering},
        mpsc::{self, Receiver, RecvError, Sender},
    },
    task::Poll,
    thread,
};

use jaypty::{io::PseudoTerminalRegisterIO, tokens::Token};
use jaysync::mpsc::PeekableReciever;
use polling::{
    Event, Poller,
    os::iocp::{CompletionPacket, PollerIocpExt},
};
use windows_sys::Win32::{
    Foundation::{BOOLEAN, HANDLE, STILL_ACTIVE},
    System::Threading::{
        GetExitCodeProcess, GetProcessId, INFINITE, RegisterWaitForSingleObject, TerminateProcess,
        UnregisterWait, WT_EXECUTEINWAITTHREAD, WT_EXECUTEONLYONCE, WaitForSingleObject,
    },
};

use crate::Result;
use crate::{Error, error::ChildError};

struct Intrest {
    event: Event,
    poller: Arc<Poller>,
}

impl Intrest {
    pub fn new(poller: &Arc<Poller>, event: Event) -> Self {
        Self {
            poller: poller.clone(),
            event,
        }
    }
}

struct SentPacket {
    queuer: Sender<Result<u32>>,
    intrest: Arc<Mutex<Option<Intrest>>>,
    child_handle: AtomicPtr<c_void>,
}

pub struct ChildWatchdog {
    pid: Option<NonZeroU32>,
    child_handle: Arc<AtomicPtr<c_void>>,
    wait_handle: AtomicPtr<c_void>,
    rx: PeekableReciever<Result<u32>>,
    exit_status: Option<Result<u32>>,
    intrest: Arc<Mutex<Option<Intrest>>>,
}

impl Drop for ChildWatchdog {
    fn drop(&mut self) {
        unsafe {
            UnregisterWait(self.wait_handle.load(Ordering::Relaxed));
        }
    }
}

impl ChildWatchdog {
    pub fn new(handle: HANDLE) -> Self {
        let (tx, rx) = mpsc::channel();

        let pid = unsafe { NonZeroU32::new(GetProcessId(handle)) };

        extern "system" fn callback(ctx: *mut c_void, timed_out: BOOLEAN) {
            if timed_out != 0 {
                return;
            }
            let packet = unsafe { Box::from_raw(ctx as *mut SentPacket) };
            let mut exit_code = 0u32;
            let child_handle = packet.child_handle.load(Ordering::Relaxed);
            unsafe {
                let exit_stat = GetExitCodeProcess(child_handle, &mut exit_code);
            }
            let lock = packet.intrest.lock().expect("unable to aquire lock");
            if let Some(intrest) = lock.as_ref() {
                intrest
                    .poller
                    .post(CompletionPacket::new(intrest.event))
                    .unwrap();
            }
            if let Err(e) = packet.queuer.send(Ok(exit_code)) {
                log::error!("found error with {}", e);
                panic!("FOUND ERROR IN CHILD WATCHDOG THREAD WHEN SENDING PACKET: ({e})")
            }
        }

        let intrest = Arc::new(Mutex::new(None));
        let sent_intrest = intrest.clone();
        let sent_queuer = tx.clone();
        let packet = Box::new(SentPacket {
            intrest: sent_intrest,
            queuer: sent_queuer,
            child_handle: AtomicPtr::from(handle),
        });

        let mut wait_handle = ptr::null_mut() as HANDLE;

        unsafe {
            if let stat = RegisterWaitForSingleObject(
                &mut wait_handle,
                handle,
                Some(callback),
                Box::into_raw(packet).cast(),
                INFINITE,
                WT_EXECUTEINWAITTHREAD | WT_EXECUTEONLYONCE,
            ) && stat == 0
            {}
        }
        Self {
            pid,
            child_handle: Arc::new(AtomicPtr::from(handle)),
            wait_handle: AtomicPtr::from(wait_handle),
            rx: PeekableReciever::new(rx),
            exit_status: None,
            intrest,
        }
    }

    pub fn child_handle(&self) -> *mut c_void {
        self.child_handle.load(Ordering::Relaxed)
    }

    pub fn kill(&self) {
        unsafe {
            TerminateProcess(self.child_handle.load(Ordering::Relaxed) as *mut c_void, 0);
        }
    }

    pub fn status(&mut self) -> Option<Result<u32>> {
        self.exit_status = self.exit_status.clone().or_else(|| self.rx.pop());
        self.exit_status.clone()
    }

    pub fn wait(&mut self) -> core::result::Result<Result<u32>, RecvError> {
        self.rx.recv()
    }
}

impl PseudoTerminalRegisterIO for ChildWatchdog {
    unsafe fn register(
        &mut self,
        poller: &Arc<Poller>,
        intrest: Event,
        _: Option<polling::PollMode>,
    ) {
        let mut lock = self.intrest.lock().unwrap();
        *lock = Some(Intrest::new(poller, Token::ChildWatchDog.keyify(intrest)))
    }

    unsafe fn reregister(
        &mut self,
        poller: &Arc<Poller>,
        intrest: Event,
        mode: Option<polling::PollMode>,
    ) {
        unsafe {
            self.register(poller, intrest, mode);
        }
    }

    unsafe fn unregister(&mut self) {
        let mut lock = self.intrest.lock().unwrap();
        *lock = None;
    }
}

impl Future for ChildWatchdog {
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.status() {
            Some(code) => {
                let waker = cx.waker().clone();
                waker.wake();
                Poll::Ready(code)
            }
            None => Poll::Pending,
        }
    }
    type Output = Result<u32>;
}
