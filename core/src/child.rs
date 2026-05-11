use std::sync::Arc;

use polling::{Event, Poller};

pub trait ChildPollRegisterIO {
    unsafe fn register(&mut self, poller: &Arc<Poller>, intrest: Event);
    unsafe fn reregister(&mut self, poller: &Arc<Poller>, intrest: Event);
    unsafe fn unregister(&mut self, poller: &Arc<Poller>);
}

pub trait ChildWatchDogIO: ChildPollRegisterIO {
    fn status(&mut self) -> Option<crate::Result<u32>>;

    fn is_dead(&self) -> bool;

    fn wait(&mut self);
}
