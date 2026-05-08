use std::sync::mpsc::{SendError, Sender};

pub struct Notifier<Event: Clone> {
    sender: Sender<Event>,
    event: Event,
}

impl<Event: Clone> Notifier<Event> {
    pub fn new(sender: Sender<Event>, event: Event) -> Self {
        Self { sender, event }
    }

    pub fn notify(&self) -> Result<(), SendError<Event>> {
        self.sender.send(self.event.clone())
    }
}
