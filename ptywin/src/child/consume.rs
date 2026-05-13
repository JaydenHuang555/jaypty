use std::{ffi::c_void, sync::atomic::AtomicPtr};

use jaypty_core::child::consume::ConsumedChildConsumer;

use crate::child::{ChildHandle, killer::ConsumedContpyChildKiller};

pub struct ConsumedContpyConsumer(pub(crate) ChildHandle);

impl ConsumedContpyConsumer {
    pub fn consume(self) -> ChildHandle {
        self.0
    }
}

impl ConsumedChildConsumer<ConsumedContpyChildKiller> for ConsumedContpyConsumer {
    fn killer(self) -> ConsumedContpyChildKiller {
        ConsumedContpyChildKiller(self.consume())
    }
}
