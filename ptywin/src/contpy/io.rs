use std::sync::mpsc::{self, Receiver, Sender};

use jaypty::{PseudoTerminalIO, event::EventKind};
use jaysync::io::nonblocking::{NonBlockingPipeReader, NonBlockingPipeWriter};
use windows_sys::Win32::System::Console::COORD;

use super::ContpyHandle;
use crate::contpy::{ContpySpawn, factory};

pub struct ContpyPseudoTerminalIO {
    tx: Sender<EventKind>,
    cout: NonBlockingPipeReader,
    cin: NonBlockingPipeWriter,
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

impl PseudoTerminalIO<NonBlockingPipeReader, NonBlockingPipeWriter> for ContpyPseudoTerminalIO {
    fn new(options: jaypty::Options, tx: Sender<EventKind>) -> Self {
        let mut spawn = ContpySpawn::spawn(options);
        let cin = factory::cin(spawn.cin.take().unwrap());
        let cout = factory::cout(spawn.cout.take().unwrap(), tx.clone(), EventKind::CoutRead);

        Self {
            tx,
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

    #[inline]
    fn cout(&mut self) -> &mut NonBlockingPipeReader {
        &mut self.cout
    }

    #[inline]
    fn cin(&mut self) -> &mut NonBlockingPipeWriter {
        &mut self.cin
    }
}
