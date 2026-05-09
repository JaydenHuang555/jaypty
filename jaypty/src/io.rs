use std::{
    io::{Read, Write},
    sync::Arc,
};

use polling::{Event, PollMode, Poller};

use crate::{Options, PtySize};

pub trait PseudoTerminalRegisterIO {
    unsafe fn register(&mut self, poller: &Arc<Poller>, intrest: Event, mode: Option<PollMode>);
    unsafe fn reregister(&mut self, poller: &Arc<Poller>, intrest: Event, mode: Option<PollMode>);
    unsafe fn unregister(&mut self);
}

pub trait PseudoTerminalIO<Reader: Read, Writer: Write>: PseudoTerminalRegisterIO {
    fn new(_options: Options) -> Self;

    fn resize(&mut self, _size: PtySize);
}
