mod os;
mod system;

pub use jaypty_core::UnDefinedPseudoTerminalIO;
pub use jaypty_core::child::ChildWatchDogIO;
pub use jaypty_core::command::Command;

pub use jaypty_core::child::*;

#[cfg(test)]
mod test {
    use std::{
        process::Command,
        sync::Arc,
        time::{Duration, Instant},
    };

    use jaypty_core::{
        Options,
        child::{ChildPollRegisterIO, ChildWatchDogIO},
    };
    use polling::{Event, Events, Poller};
    use unixpty::{ConsumedChildConsumer, ConsumedChildKiller};

    use crate::{os::SystemWatchDogIO, system::PseudoTermainalSubsystem};

    #[test]
    fn kill() {
        const KILL_TOKEN: usize = 0;

        let mut io = PseudoTermainalSubsystem::new(Options::default()).unwrap();
        let child = io.consume_child().unwrap();
        let mut watch_dog = io.latch_watchdog().unwrap();

        let poller = Arc::new(Poller::new().unwrap());
        unsafe { watch_dog.register(&poller, Event::readable(KILL_TOKEN)) }.unwrap();
        child.killer().blocking().unwrap();

        let mut events = Events::new();
        poller
            .wait(&mut events, Some(Duration::from_millis(200)))
            .unwrap();
        unsafe { watch_dog.unregister(&poller).unwrap() };
        for event in events.iter() {
            if event.key == KILL_TOKEN && event.readable {
                return;
            }
        }
        panic!("failed to kill child")
    }
}
