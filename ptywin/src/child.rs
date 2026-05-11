use std::{
    ffi::c_void,
    future::Pending,
    num::NonZeroU32,
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicPtr, Ordering},
        mpsc::{self, Receiver, RecvError, Sender, TryRecvError},
    },
    task::Poll,
    thread,
    time::Duration,
};

use jaypty_core::{
    EmptyResult,
    child::{ChildPollRegisterIO, ChildStatus, ChildWatchDogIO},
};
use jaypty_error::{OsEmptyResult, OsError, OsResult, SystemError};
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
    queuer: Sender<ChildStatus>,
    intrest: Arc<Mutex<Option<Intrest>>>,
    child_handle: AtomicPtr<c_void>,
}

pub struct WinChildWatchdogIO {
    pid: Option<NonZeroU32>,
    child_handle: Arc<AtomicPtr<c_void>>,
    wait_handle: AtomicPtr<c_void>,
    tx: Sender<ChildStatus>,
    rx: Receiver<ChildStatus>,
    exit_status: ChildStatus,
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
    pub fn latch(handle: HANDLE) -> OsResult<Self> {
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
            if let Err(e) = packet.queuer.send(ChildStatus::Dead(exit_code)) {
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
        Ok(Self {
            pid,
            child_handle: Arc::new(AtomicPtr::from(handle)),
            wait_handle: AtomicPtr::from(wait_handle),
            exit_status: ChildStatus::Alive,
            rx,
            intrest,
            tx,
        })
    }
}

impl ChildPollRegisterIO<OsError> for WinChildWatchdogIO {
    unsafe fn register(&mut self, poller: &Arc<Poller>, intrest: Event) -> OsEmptyResult {
        let mut lock = self
            .intrest
            .lock()
            .map_err(|_| SystemError::ChildWatchDogRegisterMutexPoison)?;
        *lock = Some(Intrest {
            event: intrest,
            poller: poller.clone(),
        });
        Ok(())
    }

    unsafe fn reregister(&mut self, poller: &Arc<Poller>, intrest: Event) -> OsEmptyResult {
        let mut lock = self
            .intrest
            .lock()
            .map_err(|_| SystemError::ChildWatchDogRegisterMutexPoison)?;
        *lock = Some(Intrest {
            event: intrest,
            poller: poller.clone(),
        });
        Ok(())
    }

    unsafe fn unregister(&mut self, _: &Arc<Poller>) -> OsEmptyResult {
        let mut lock = self
            .intrest
            .lock()
            .map_err(|_| SystemError::ChildWatchDogRegisterMutexPoison)?;
        *lock = None;
        Ok(())
    }
}

impl ChildWatchDogIO<OsError> for WinChildWatchdogIO {
    fn status(&mut self) -> OsResult<ChildStatus> {
        if self.exit_status.is_dead() {
            Ok(self.exit_status)
        } else {
            match self.rx.try_recv() {
                Ok(stat) => {
                    self.exit_status = stat;
                    Ok(self.exit_status)
                }
                Err(TryRecvError::Disconnected) => {
                    Err(SystemError::ChildWatchDogChannelDisconnected)
                }

                Err(TryRecvError::Empty) => Ok(self.exit_status),
            }
        }
    }

    fn is_dead(&self) -> OsResult<ChildStatus> {
        todo!()
    }

    fn wait(&mut self) -> OsResult<ChildStatus> {
        todo!()
    }
}
