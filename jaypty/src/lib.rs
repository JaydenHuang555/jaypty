mod os;

pub use jaypty_core::PseudoTerminalIO;
pub use jaypty_core::child::ChildWatchDogIO;
pub use jaypty_core::command::Command;
pub use jaypty_core::io::PollingIntrestRegisterIO;
pub use jaypty_core::{Options, PtySize};

pub use os::{SystemPseudoTerminalIO, SystemWatchDogIO};

pub use polling::Event as PollIntrest;
pub use polling::Events as PolledEvents;
pub use polling::PollMode;
pub use polling::Poller;

#[cfg(test)]
mod tests {
    use crate::SystemPseudoTerminalIO;

    // just a simple check to make sure it can compile on systems
    #[test]
    fn system() {
        let io = SystemPseudoTerminalIO::default();
    }
}
