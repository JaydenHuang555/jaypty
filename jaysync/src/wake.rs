use std::{sync::mpsc::Sender, task::Wake, thread};

pub struct ThreadWaker(pub thread::Thread);

impl Wake for ThreadWaker {
    fn wake(self: std::sync::Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &std::sync::Arc<Self>) {
        self.0.unpark();
    }
}

pub struct EventSenderWaker<Event: Clone> {
    sender: Sender<Event>,
    event: Event,
}

impl<Event: Clone> EventSenderWaker<Event> {
    pub fn new(sender: Sender<Event>, event: Event) -> Self {
        Self { sender, event }
    }
}

impl<Event: Clone> Wake for EventSenderWaker<Event> {
    fn wake(self: std::sync::Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &std::sync::Arc<Self>) {
        self.sender.send(self.event.clone()).unwrap();
    }
}
