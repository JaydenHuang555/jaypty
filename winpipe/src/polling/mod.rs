pub mod read;
pub mod write;

pub use read::PollingWakingNonBlockingPipeReader;
pub use write::PollingWakingNonBlockingPipeWriter;

use std::{
    sync::{Arc, Mutex},
    task::Wake,
};

use polling::{
    Event, PollMode, Poller,
    os::iocp::{CompletionPacket, PollerIocpExt},
};

pub(crate) struct Polled {
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

pub(crate) struct RegisteredPoll {
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
        let mut lock = self
            .polled
            .lock()
            .expect("mutex lock for registered poll is poisoned");
        if let Some(intrest) = lock.as_ref() {
            intrest
                .poller
                .post(CompletionPacket::new(intrest.event))
                .unwrap();
            if intrest
                .mode
                .is_some_and(|m| m == PollMode::Oneshot || m == PollMode::EdgeOneshot)
            {
                *lock = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::Arc,
        task::Waker,
        time::{Duration, Instant},
    };

    use polling::{Event, Events, Poller};

    use crate::polling::{Polled, RegisteredPoll};

    #[test]
    fn wakey_poll_test() {
        const SENT_EVENT: Event = Event::readable(0);
        const TIMEOUT: Duration = Duration::from_millis(200);

        let poller = Arc::new(Poller::new().expect("failed to create poller"));
        let poll = Arc::new(RegisteredPoll::default());
        {
            let mut lock = poll.polled.lock().unwrap();
            *lock = Some(Polled::new(&poller, SENT_EVENT, None));
            drop(lock)
        }

        let waker = Waker::from(poll);
        waker.wake();
        let mut events = Events::new();
        poller
            .wait_deadline(&mut events, Instant::now() + TIMEOUT)
            .unwrap();
        assert_eq!(events.iter().next().unwrap(), SENT_EVENT)
    }
}
