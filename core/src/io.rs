use std::{
    io::{Read, Write},
    sync::Arc,
};

use polling::{Event, PollMode, Poller};

use crate::Result;
use crate::{Options, PtySize, child::ChildWatchDogIO};

pub trait PollingIntrestRegisterIO {
    unsafe fn register(&mut self, poller: &Arc<Poller>, intrest: Event, mode: Option<PollMode>);
    unsafe fn reregister(&mut self, poller: &Arc<Poller>, intrest: Event, mode: Option<PollMode>);
    unsafe fn unregister(&mut self);
}

pub trait PseudoTerminalIO<R: Read, W: Write, ChildWatchdog: ChildWatchDogIO>:
    PollingIntrestRegisterIO + Write + Read + Default
{
    fn new(_options: Options) -> Self;

    fn resize(&mut self, _size: PtySize);
    fn spawn_and_latch_child_watchdog(&self) -> ChildWatchdog;
    fn kill_child(&mut self) -> Result<()>;

    fn cin(&mut self) -> &mut W;
    fn cout(&mut self) -> &mut R;
}
