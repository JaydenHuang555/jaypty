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
use jaysync::io::{ReadEventCapture, WriteEventCapture, WriteEvents};
use polling::{Event, Events, PollMode, Poller};
use ptywin::contpy::ContpyIO;
use ptywin::pipe::ScheduledEvent;
use ptywin::pipe::input::NonBlockingPipeWriter;
use ptywin::pipe::output::NonBlockingPipeReader;
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
    let cout = ReadEventCapture::new(
        NonBlockingPipeReader::new(io.take_cout(), 1024),
        tx.clone(),
        EventKind::CoutRead,
    );
    let mut cin = WriteEventCapture::new(
        NonBlockingPipeWriter::new(io.take_cin(), 1024),
        tx.clone(),
        WriteEvents {
            write_event: Some(EventKind::CinWrite),
            ..Default::default()
        },
    );

    thread::spawn(move || {
        loop {
            let rec = rx.recv().unwrap();
            log::info!("found event: {:?}", rec);
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
