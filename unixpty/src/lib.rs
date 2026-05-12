#[cfg(unix)]
pub mod child;
#[cfg(unix)]
mod defs;
#[cfg(unix)]
pub mod err;
#[cfg(unix)]
pub(crate) mod factory;
#[cfg(unix)]
pub(crate) mod io;
#[cfg(unix)]
pub(crate) mod uses;

#[cfg(unix)]
pub use uses::*;

#[cfg(all(test, unix))]
mod test {
    use std::{sync::Arc, time::Duration};

    use jaypty_core::{Options, UnDefinedPseudoTerminalIO, child::ChildPollRegisterIO};
    use polling::{Event, Events, Poller};

    use crate::UnixPseudoTerminalIO;

    #[test]
    fn drop_child() {
        const EXIT_INTRESTS: Event = Event::readable(0);
        const TIMEOUT: Duration = Duration::from_millis(500);

        let io = UnixPseudoTerminalIO::new(Options::default()).expect("unable to write thread");
        let mut watch_dog = io.latch_watchdog().expect("failed to latch watchdog");

        let poller = Arc::new(Poller::new().unwrap());

        unsafe { watch_dog.register(&poller, EXIT_INTRESTS) }.unwrap();

        drop(io);

        let mut events = Events::new();

        events.clear();
        poller.wait(&mut events, Some(TIMEOUT)).unwrap();
        for event in events.iter() {
            if event.readable && event.key == EXIT_INTRESTS.key {
                unsafe { watch_dog.unregister(&poller).unwrap() }
                return;
            }
        }
        panic!("child did not drop")
    }
}
