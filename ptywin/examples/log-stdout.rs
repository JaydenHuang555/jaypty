use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::num::NonZeroUsize;
use std::panic::resume_unwind;
use std::sync::{Arc, RwLock, mpsc};
use std::thread;
use std::time::Duration;

use env_logger::Builder;
use jaypty::PtySize;
use jaypty::event::EventKind;
use jaypty::tokens::{PtyTokens, TOKEN_READ, TOKEN_WRITE};
use jaysync::io::nonblocking::{NonBlockingPipeReader, NonBlockingPipeWriter};
use jaysync::io::{ReadEventCapture, WriteEventCapture, WriteEvents};
use polling::{Event, Events, PollMode, Poller};
use ptywin::contpy::ContpyIO;
use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole, FreeConsole};

#[derive(Clone, Debug)]
pub struct State {
    built_content: String,
}

pub fn main() {
    let mut b = Builder::new();
    b.filter_level(log::LevelFilter::Info);
    b.init();
    let mut io = ContpyIO::new(PtySize {
        columns: 24,
        rows: 80,
    });
    let (tx, rx) = mpsc::channel();

    let cout_capture = ReadEventCapture::new(io.take_cout(), tx.clone(), EventKind::CoutRead);
    let cin_capture = WriteEventCapture::new(
        io.take_cin(),
        tx.clone(),
        WriteEvents {
            write_event: Some(EventKind::CinWrite),
            ..Default::default()
        },
    );

    let mut cout = NonBlockingPipeReader::new(cout_capture, 1024);
    let mut cin = NonBlockingPipeWriter::new(cin_capture, 1024);

    thread::spawn(move || {
        let mut relay = BufWriter::new(File::create("RELAY").unwrap());
        loop {
            let rec: EventKind = rx.recv().unwrap();
            log::info!("found event: {:?}", rec);

            if rec == EventKind::CoutRead {
                let mut buff = [0u8; 512];
                loop {
                    match cout.read(&mut buff) {
                        Ok(count) => {
                            if count == 0 {
                                break;
                            }
                            relay.write(&buff[..count]).unwrap();
                            relay.flush().unwrap();
                        }
                        Err(e) => {
                            log::error!("error when reading from cout {}", e);
                            break;
                        }
                    }
                }
            }
        }
    });

    loop {
        println!("please enter v command");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let content = String::from(line.trim_end());

        if content == "exit" {
            break;
        }

        let bytes = content.as_bytes();
        cin.write(bytes).unwrap();
        cin.write(b"\n").unwrap();
    }
}
