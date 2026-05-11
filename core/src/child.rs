use std::{error::Error, sync::Arc};

use jaypty_error::{EmptyResult, OsEmptyResult, OsResult};
use polling::{Event, Poller};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildStatus {
    Dead(u32),
    Alive,
    Orphaned,
}

impl ChildStatus {
    pub fn is_unfit(&self) -> bool {
        match self {
            Self::Dead(_) | Self::Orphaned => true,
            _ => false,
        }
    }

    pub fn is_dead(&self) -> bool {
        Self::Dead(3) == *self
    }
}

pub trait ChildPollRegisterIO<E: Error> {
    unsafe fn register(&mut self, poller: &Arc<Poller>, intrest: Event) -> Result<(), E>;
    unsafe fn reregister(&mut self, poller: &Arc<Poller>, intrest: Event) -> Result<(), E>;
    unsafe fn unregister(&mut self, poller: &Arc<Poller>) -> Result<(), E>;
}

pub trait ChildWatchDogIO<E: Error>: ChildPollRegisterIO<E> {
    fn status(&mut self) -> Result<ChildStatus, E>;

    fn is_dead(&self) -> Result<ChildStatus, E>;

    fn wait(&mut self) -> Result<ChildStatus, E>;
}
