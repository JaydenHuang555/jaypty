#[cfg(windows)]
pub mod child;
#[cfg(windows)]
pub mod poll;

#[cfg(windows)]
pub mod error;
#[cfg(windows)]
pub mod factory;
#[cfg(windows)]
pub mod io;
#[cfg(windows)]
pub mod symbols;
#[cfg(windows)]
pub(crate) mod uses;
#[cfg(windows)]
pub(crate) mod util;

#[cfg(windows)]
pub(crate) use uses::*;

#[cfg(all(test, windows))]
mod tests {

    use std::{
        ops::Add,
        os::windows::{io::AsRawHandle, raw::HANDLE},
        sync::Arc,
        time::{Duration, Instant},
    };

    use jaypty_core::child::ChildPollRegisterIO;
    use polling::{Event, Events, Poller};

    use crate::child::WinChildWatchdogIO;

    #[test]
    pub fn child() {
        const POLLING_TIMEOUT: Duration = Duration::from_millis(400);

        let mut child = std::process::Command::new("cmd.exe").spawn().unwrap();
        let mut watchdog = WinChildWatchdogIO::latch(child.as_raw_handle() as HANDLE);
        let poller = Arc::new(Poller::new().unwrap());
        unsafe {
            watchdog.register(&poller, Event::readable(0));
        }
        let mut events = Events::new();
        child.kill().expect("error when killing child");

        poller
            .wait_deadline(&mut events, Instant::now().add(POLLING_TIMEOUT))
            .expect("found error when polling");
        assert_eq!(events.iter().next().is_some(), true);
    }
}
