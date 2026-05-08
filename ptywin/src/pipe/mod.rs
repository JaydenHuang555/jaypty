use std::{
    io::Write,
    marker::PhantomData,
    sync::{Arc, Mutex, RwLock, mpsc::Sender},
    task::{Context, Poll, Wake, Waker},
    thread,
};

pub mod input;
pub mod output;

use jaypty::pipe::PipeKind;
use jaysync::queue::ScheduledEventMode;
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
pub struct ScheduledEvent<Event: Clone> {
    pub event: Event,
    pub mode: ScheduledEventMode,
}

impl<E: Clone> From<E> for ScheduledEvent<E> {
    fn from(value: E) -> Self {
        Self::new(value, None)
    }
}

impl<Event: Clone> ScheduledEvent<Event> {
    pub fn new(event: Event, mode: Option<ScheduledEventMode>) -> Self {
        Self {
            event,
            mode: mode.unwrap_or_default(),
        }
    }
}

impl<Event: Clone + Default> Default for ScheduledEvent<Event> {
    fn default() -> Self {
        Self {
            event: Event::default(),
            mode: ScheduledEventMode::default(),
        }
    }
}

pub struct Task<Event: Clone> {
    pub sender: RwLock<Option<Sender<Event>>>,
    pub event: Mutex<Option<ScheduledEvent<Event>>>,
}

impl<Event: Clone> Task<Event> {
    pub fn new() -> Self {
        Self {
            sender: RwLock::new(None),
            event: Mutex::new(None),
        }
    }

    pub fn update_sender(&mut self, sender: impl Into<Option<Sender<Event>>>) {
        let mut write = self.sender.write().unwrap();
        *write = sender.into();
    }

    pub fn update_event(&mut self, event: impl Into<Option<ScheduledEvent<Event>>>) {
        let mut lock = self.event.lock().unwrap();
        *lock = event.into()
    }
}

impl<Event: Clone> Default for Task<Event> {
    fn default() -> Self {
        Self {
            sender: RwLock::new(None),
            event: Mutex::new(None),
        }
    }
}

impl<Event: Clone> Wake for Task<Event> {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let mut lock = self.event.lock().unwrap();
        if let Some(event) = lock.clone() {
            let sender_lock = self.sender.read().unwrap();
            sender_lock.as_ref().map(|sender| sender.send(event.event));
            if event.mode == ScheduledEventMode::Instant {
                *lock = None;
            }
        }
    }
}
