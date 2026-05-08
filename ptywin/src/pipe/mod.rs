use std::{
    io::Write,
    marker::PhantomData,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
    thread,
};

pub mod input;
pub mod output;

use jaypty::{message::Message, pipe::PipeKind};
use miow::pipe::AnonWrite;
use piper::{Reader, Writer, pipe};
use polling::{
    Event, PollMode, Poller,
    os::iocp::{CompletionPacket, PollerIocpExt},
};
use windows_sys::Win32::Foundation::HANDLE;

pub struct ThreadWaker(pub thread::Thread);

impl Wake for ThreadWaker {
    fn wake(self: std::sync::Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &std::sync::Arc<Self>) {
        self.0.unpark();
    }
}

#[derive(Clone)]
pub struct RegisteredTask {
    poller: Arc<Poller>,
    event: Event,
    mode: PollMode,
}

impl RegisteredTask {
    pub fn new(poller: Arc<Poller>, event: Event, mode: PollMode) -> Self {
        Self {
            poller,
            event,
            mode,
        }
    }
}

pub struct WrappedRegisteredTask {
    kind: PipeKind,
    task: Mutex<Option<RegisteredTask>>,
}

impl WrappedRegisteredTask {
    pub fn new(kind: PipeKind) -> Self {
        Self {
            kind,
            task: Mutex::new(None),
        }
    }

    pub fn can_run(&self, event: Event) -> bool {
        event.readable == self.kind.is_readible()
    }
}

impl Wake for WrappedRegisteredTask {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let mut task_lock = self.task.lock().unwrap();
        if let Some(task) = task_lock.as_ref() {
            if self.can_run(task.event) {
                task.poller.post(CompletionPacket::new(task.event)).ok();
                if task.mode == PollMode::Oneshot || task.mode == PollMode::EdgeOneshot {
                    *task_lock = None;
                }
            }
        }
    }
}
