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
    time::Duration,
};

use jaypty_core::{
    child::{ChildPollRegisterIO, ChildWatchDogIO},
    tokens::Token,
};
use jaysync::{mpsc::PeekableReciever, wake::ThreadWaker};
use polling::{
    Event, Poller,
    os::iocp::{CompletionPacket, PollerIocpExt},
};
use windows_sys::Win32::{
    Foundation::{BOOLEAN, HANDLE, INVALID_HANDLE_VALUE, STILL_ACTIVE},
    System::Threading::{
        GetExitCodeProcess, GetProcessId, INFINITE, RegisterWaitForSingleObject, TerminateProcess,
        UnregisterWait, UnregisterWaitEx, WT_EXECUTEINWAITTHREAD, WT_EXECUTEONLYONCE,
        WaitForSingleObject,
    },
};

use crate::Result;
use crate::{Error, error::ChildError};

#[derive(Clone, Debug)]
pub enum Message {
    ChildExit(Result<u32>),
    WaitUnRegistered(i32),
}

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
    queuer: Sender<Message>,
    intrest: Arc<Mutex<Option<Intrest>>>,
    child_handle: AtomicPtr<c_void>,
}

pub struct WinChildWatchdogIO {
    pid: Option<NonZeroU32>,
    child_handle: Arc<AtomicPtr<c_void>>,
    wait_handle: AtomicPtr<c_void>,
    tx: Sender<Message>,
    rx: PeekableReciever<Message>,
    exit_status: Option<Result<u32>>,
    intrest: Arc<Mutex<Option<Intrest>>>,
}

impl Drop for WinChildWatchdogIO {
    fn drop(&mut self) {
        let _ = unsafe {
            UnregisterWaitEx(
                self.wait_handle.load(Ordering::Relaxed),
                INVALID_HANDLE_VALUE,
            )
        };
    }
}

impl WinChildWatchdogIO {
    pub fn latch(handle: HANDLE) -> Self {
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
                if exit_stat == STILL_ACTIVE {
                    packet
                        .queuer
                        .send(Message::ChildExit(Err(Error::kind(
                            ChildError::AliveDuringExitCallBack,
                        )
                        .context("the child is still alive on the exit call back"))))
                        .expect("unable to send fail packet");
                    return;
                }
            }
            let lock = packet.intrest.lock().expect("unable to aquire lock");
            if let Some(intrest) = lock.as_ref() {
                intrest
                    .poller
                    .post(CompletionPacket::new(intrest.event))
                    .unwrap();
            }
            if let Err(e) = packet.queuer.send(Message::ChildExit(Ok(exit_code))) {
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
            tx,
        }
    }
}

impl ChildPollRegisterIO for WinChildWatchdogIO {
    unsafe fn register(&mut self, poller: &Arc<Poller>, intrest: Event) {
        let mut lock = self.intrest.lock().unwrap();
        *lock = Some(Intrest {
            event: intrest,
            poller: poller.clone(),
        })
    }

    unsafe fn reregister(&mut self, poller: &Arc<Poller>, intrest: Event) {
        let mut lock = self.intrest.lock().unwrap();
        *lock = Some(Intrest {
            event: intrest,
            poller: poller.clone(),
        })
    }

    unsafe fn unregister(&mut self, _: &Arc<Poller>) {
        let mut lock = self.intrest.lock().unwrap();
        *lock = None;
    }
}

impl ChildWatchDogIO for WinChildWatchdogIO {
    fn status(&mut self) -> Option<jaypty_core::Result<u32>> {
        self.exit_status = self.exit_status.take().or_else(|| {
            self.rx
                .pop()
                .map(|m| match m {
                    Message::ChildExit(stat) => Some(stat),
                    _ => None,
                })
                .unwrap_or(None)
        });
        self.exit_status.clone()
    }

    fn is_dead(&self) -> bool {
        self.exit_status.is_some()
    }

    fn wait(&mut self) {
        loop {
            match self.status() {
                None => {}
                Some(_) => break,
            }
            thread::sleep(Duration::from_millis(300));
        }
    }
}

impl Future for WinChildWatchdogIO {
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match self.status() {
            None => Poll::Pending,
            Some(_) => {
                let waker = cx.waker().clone();
                waker.wake();
                Poll::Ready(Ok(()))
            }
        }
    }

    type Output = Result<()>;
}
