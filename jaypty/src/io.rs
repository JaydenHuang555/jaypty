use std::{
    io::{Read, Write},
    marker::PhantomData,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::{Options, PtySize, event::EventKind};

pub trait PseudoTerminalIO<Reader: Read, Writer: Write> {
    fn new(_options: Options, sender: Sender<EventKind>) -> Self;

    fn resize(&mut self, _size: PtySize);

    fn cout(&mut self) -> &mut Reader;
    fn cin(&mut self) -> &mut Writer;
}

pub fn factory<Reader: Read, Writer: Write, IO: PseudoTerminalIO<Reader, Writer> + 'static>(
    _io: PhantomData<IO>,
    options: Options,
) -> (IO, Receiver<EventKind>) {
    let (tx, rx) = mpsc::channel();
    let io = IO::new(options, tx);
    (io, rx)
}
