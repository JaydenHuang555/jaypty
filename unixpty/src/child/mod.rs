pub mod killer;
pub mod watchdog;

use std::process::Child;

use jaypty_core::child::consume::ConsumedChildConsumer;
pub use killer::*;
pub use watchdog::*;

use crate::child::killer::ConsumedPosixChildKiller;

pub struct ConsumedPosixChildConsumer(pub(crate) Child);

impl ConsumedChildConsumer<ConsumedPosixChildKiller> for ConsumedPosixChildConsumer {
    fn killer(self) -> ConsumedPosixChildKiller {
        ConsumedPosixChildKiller(self.0)
    }
}
