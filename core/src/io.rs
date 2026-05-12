use std::{
    error::Error,
    io::{Read, Write},
    result,
    sync::Arc,
};

use polling::{Event, PollMode, Poller};

use jaypty_error::{OsEmptyResult, OsResult};

use crate::{
    EmptyResult, Options, PtySize,
    child::{ChildWatchDogIO, consume::ConsumedChildConsumer, killer::ConsumedChildKiller},
};

pub trait PollingIntrestRegisterIO<E: Error> {
    unsafe fn register(
        &mut self,
        poller: &Arc<Poller>,
        intrest: Event,
        mode: Option<PollMode>,
    ) -> result::Result<(), E>;
    fn reregister(
        &mut self,
        poller: &Arc<Poller>,
        intrest: Event,
        mode: Option<PollMode>,
    ) -> result::Result<(), E>;
    fn unregister(&mut self, poller: &Arc<Poller>) -> result::Result<(), E>;
}

pub trait UnDefinedPseudoTerminalIO<
    R: Read,
    W: Write,
    WatchDog: ChildWatchDogIO<E>,
    E: Error,
    ChildKiller: ConsumedChildKiller,
    ConsChild: ConsumedChildConsumer<ChildKiller>,
>: PollingIntrestRegisterIO<E> + Write + Read where
    Self: Sized,
{
    fn new(_options: Options) -> Result<Self, E>;

    fn resize(&mut self, _size: PtySize) -> Result<(), E>;
    fn latch_watchdog(&self) -> Result<WatchDog, E>;
    fn kill_child(&mut self) -> Result<(), E>;

    fn cin(&mut self) -> &mut W;
    fn cout(&mut self) -> &mut R;

    fn consume_child(&mut self) -> Option<ConsChild>;
}
