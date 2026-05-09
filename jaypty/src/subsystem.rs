use std::{
    io::{Read, Write},
    marker::PhantomData,
    sync::mpsc::Receiver,
};

use crate::{Options, PseudoTerminalIO, event::EventKind, io};

pub struct PseudoTerminalSubsystem<Reader: Read, Writer: Write, IO>
where
    IO: PseudoTerminalIO<Reader, Writer>,
{
    io: IO,
    rx: Receiver<EventKind>,
    _reader: PhantomData<Reader>,
    _writer: PhantomData<Writer>,
}

impl<Reader: Read, Writer: Write, IO: PseudoTerminalIO<Reader, Writer> + 'static>
    PseudoTerminalSubsystem<Reader, Writer, IO>
{
    pub fn new(options: Options) -> Self {
        let (io, rx) = io::factory(PhantomData::<IO>, options);

        Self {
            io,
            rx,
            _reader: PhantomData,
            _writer: PhantomData,
        }
    }
}
