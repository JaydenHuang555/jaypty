pub mod child;
pub mod poll;

pub mod error;
pub mod factory;
pub mod io;
pub mod symbols;

pub(crate) use symbols::ContpyHandle;
pub(crate) use symbols::ContpySymbols;
pub(crate) use symbols::loaded_symbols;

#[cfg(not(windows))]
compile_error!("PLEASE COMPILE ON WINDOWS!");

pub use poll::RegisteredPoll;

pub use crate::factory::ContpySpawn;
pub use jaypty::error::Error;
pub use jaypty::error::ErrorKind;
pub use jaypty::error::Result;

#[cfg(test)]
mod tests {

    use std::{
        ops::Add,
        os::windows::{io::AsRawHandle, raw::HANDLE},
        sync::Arc,
        time::{Duration, Instant},
    };

    use jaypty::{
        io::{SafePseudoTerminalRegisterIO, UnsafePseudoTerminalRegisterIO},
        tokens::Token,
    };
    use jaysync::io::{
        ReadEventCapture, WriteEventCapture, WriteEvents,
        nonblocking::{NonBlockingPipeReader, NonBlockingPipeWriter},
    };
    use polling::{Event, Events, Poller};

    use crate::child::WinChildWatchdogIO;

    #[test]
    pub fn child() {
        const POLLING_TIMEOUT: Duration = Duration::from_millis(400);

        let mut child = std::process::Command::new("cmd.exe").spawn().unwrap();
        let mut watchdog = WinChildWatchdogIO::latch(child.as_raw_handle() as HANDLE);
        let poller = Arc::new(Poller::new().unwrap());
        watchdog.register(&poller, None);
        let mut events = Events::new();
        child.kill().expect("error when killing child");

        poller
            .wait_deadline(&mut events, Instant::now().add(POLLING_TIMEOUT))
            .expect("found error when polling");
        assert_eq!(events.iter().next().is_some(), true);
    }
}
