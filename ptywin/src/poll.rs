use std::{
    sync::{Arc, Mutex},
    task::Wake,
};

use polling::{
    Event, PollMode, Poller,
    os::iocp::{CompletionPacket, PollerIocpExt},
};

pub struct Polled {
    pub poller: Arc<Poller>,
    pub event: Event,
    pub mode: Option<PollMode>,
}

impl Polled {
    pub fn new(poller: &Arc<Poller>, event: Event, mode: Option<PollMode>) -> Self {
        Self {
            poller: poller.clone(),
            event,
            mode,
        }
    }
}

pub struct RegisteredPoll {
    pub polled: Mutex<Option<Polled>>,
}

impl Default for RegisteredPoll {
    fn default() -> Self {
        Self {
            polled: Mutex::new(None),
        }
    }
}

impl Wake for RegisteredPoll {
    fn wake(self: std::sync::Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &std::sync::Arc<Self>) {
        let lock = self
            .polled
            .lock()
            .expect("mutex lock for registered poll is poisoned");
        if let Some(intrest) = lock.as_ref() {
            intrest
                .poller
                .post(CompletionPacket::new(intrest.event))
                .unwrap();
        }
    }
}
