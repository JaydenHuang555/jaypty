use std::sync::mpsc::{Receiver, RecvError, TryRecvError};

pub struct PeekableReciever<Event> {
    reciever: Receiver<Event>,
    stored_peek: Option<Event>,
}

impl<Event> From<Receiver<Event>> for PeekableReciever<Event> {
    fn from(value: Receiver<Event>) -> Self {
        Self {
            reciever: value,
            stored_peek: None,
        }
    }
}

impl<Event: Clone> PeekableReciever<Event> {
    pub fn new(reciever: Receiver<Event>) -> Self {
        Self {
            reciever,
            stored_peek: None,
        }
    }

    pub fn peek(&mut self) -> Option<&Event> {
        self.stored_peek = self
            .stored_peek
            .clone()
            .map(|stored| Some(stored))
            .unwrap_or_else(|| match self.reciever.try_recv() {
                Ok(peeked) => Some(peeked),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => panic!("channel was closed"),
            });
        self.stored_peek.as_ref()
    }

    pub fn pop(&mut self) -> Option<Event> {
        self.stored_peek.take()
    }

    pub fn recv(&mut self) -> Result<Event, RecvError> {
        if let Some(peek) = self.stored_peek.take() {
            Ok(peek)
        } else {
            self.reciever.recv()
        }
    }
}
