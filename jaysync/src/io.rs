use std::{
    io::{Read, Write},
    sync::mpsc::Sender,
};

pub mod nonblocking;

use crate::{notifier::Notifier, wake::EventSenderWaker};

#[derive(Clone)]
pub struct WriteEvents<Event> {
    pub write_event: Option<Event>,
    pub flush_event: Option<Event>,
}

impl<E> Default for WriteEvents<E> {
    fn default() -> Self {
        Self {
            write_event: None,
            flush_event: None,
        }
    }
}

#[derive(Clone)]
pub struct WriteEventCapture<Writer: Write, Event: Clone> {
    sender: Sender<Event>,
    writer: Writer,
    events: WriteEvents<Event>,
}

impl<Writer: Write, Event: Clone> WriteEventCapture<Writer, Event> {
    pub fn new(
        writer: Writer,
        tx: Sender<Event>,
        events: impl Into<Option<WriteEvents<Event>>>,
    ) -> Self {
        Self {
            sender: tx,
            writer,
            events: events.into().unwrap_or_default(),
        }
    }
}

impl<Writer: Write, Event: Clone> Write for WriteEventCapture<Writer, Event> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let ret = self.writer.write(buf);

        if let Ok(count) = ret
            && count > 0
        {
            if let Some(write_event) = self.events.write_event.clone() {
                self.sender.send(write_event).unwrap();
            }
        }

        ret
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let ret = self.writer.flush();
        if ret.is_ok() {
            if let Some(flush_event) = self.events.flush_event.clone() {
                self.sender.send(flush_event).unwrap();
            }
        }
        ret
    }
}

#[derive(Clone)]
pub struct ReadEventCapture<Reader: Read, Event: Clone> {
    sender: Sender<Event>,
    reader: Reader,
    read_event: Event,
}

impl<Reader: Read, Event: Clone> ReadEventCapture<Reader, Event> {
    pub fn new(reader: Reader, tx: Sender<Event>, read_event: impl Into<Event>) -> Self {
        Self {
            sender: tx,
            reader,
            read_event: read_event.into(),
        }
    }
}

impl<Reader: Read, Event: Clone> Read for ReadEventCapture<Reader, Event> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let ret = self.reader.read(buf);
        if let Ok(count) = ret
            && count > 0
        {
            self.sender.send(self.read_event.clone()).unwrap();
        }
        ret
    }
}
