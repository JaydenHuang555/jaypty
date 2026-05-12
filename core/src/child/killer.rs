use std::{task::Wake, thread::JoinHandle};

use jaypty_error::Result;

use crate::child::ChildStatus;

pub trait ConsumedChildKiller {
    fn blocking(self) -> Result<ChildStatus>;
    fn nonblocking(self) -> JoinHandle<Result<ChildStatus>>;
}

pub trait OwnedChildKiller<ChildHandle>: Sized {
    fn wait(&self);

    fn take(self) -> Option<ChildHandle>;
}
