use std::{
    sync::mpsc::{self, Receiver, Sender},
    task::Wake,
};

use jaypty::{PseudoTerminalIO, io::PseudoTerminalRegisterIO, tokens::Token};
use jaysync::io::{
    nonblocking::{NonBlockingPipeReader, NonBlockingPipeWriter},
    waking::{WakingNonBlockingPipeReader, WakingNonBlockingPipeWriter},
};
use polling::Event;
use windows_sys::Win32::System::Console::COORD;

use super::ContpyHandle;
use crate::factory;
use crate::factory::ContpySpawn;
use crate::{RegisteredPoll, poll::Polled};

pub struct ContpyPseudoTerminalIO {
    cout: WakingNonBlockingPipeReader<RegisteredPoll>,
    cin: WakingNonBlockingPipeWriter<RegisteredPoll>,
    handle: ContpyHandle,
}

impl Drop for ContpyPseudoTerminalIO {
    fn drop(&mut self) {
        unsafe {
            super::loaded_symbols().close(self.handle);
        }
    }
}

unsafe impl Send for ContpyPseudoTerminalIO {}

impl PseudoTerminalRegisterIO for ContpyPseudoTerminalIO {
    unsafe fn register(
        &mut self,
        poller: &std::sync::Arc<polling::Poller>,
        intrest: Event,
        mode: Option<polling::PollMode>,
    ) {
        self.cout.map_wake(|wake| {
            let mut lock = wake.polled.lock().unwrap();
            *lock = Some(Polled::new(poller, Token::CoutWrite.keyify(intrest), mode))
        });
        self.cin.map_wake(|wake| {
            let mut lock = wake.polled.lock().unwrap();
            *lock = Some(Polled::new(poller, Token::CinRead.keyify(intrest), mode))
        });
    }

    unsafe fn unregister(&mut self) {
        self.cin.map_wake(|wake| {
            let mut lock = wake.polled.lock().unwrap();
            *lock = None;
        });
        self.cout.map_wake(|wake| {
            let mut lock = wake.polled.lock().unwrap();
            *lock = None;
        });
    }

    unsafe fn reregister(
        &mut self,
        poller: &std::sync::Arc<polling::Poller>,
        intrest: Event,
        mode: Option<polling::PollMode>,
    ) {
        unsafe {
            self.register(poller, intrest, mode);
        }
    }
}

impl
    PseudoTerminalIO<
        WakingNonBlockingPipeReader<RegisteredPoll>,
        WakingNonBlockingPipeWriter<RegisteredPoll>,
    > for ContpyPseudoTerminalIO
{
    fn new(options: jaypty::Options) -> Self {
        let mut spawn = ContpySpawn::spawn(options);
        let cin = factory::cin(&mut spawn);
        let cout = factory::cout(&mut spawn);

        Self {
            cin,
            cout,
            handle: spawn.handle.take().expect("unable to take handle"),
        }
    }

    fn resize(&mut self, size: jaypty::PtySize) {
        unsafe {
            super::loaded_symbols().resize(
                self.handle,
                COORD {
                    X: size.columns as i16,
                    Y: size.rows as i16,
                },
            );
        }
    }
}
