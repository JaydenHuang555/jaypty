use std::{
    fs::File,
    io::{BufRead, BufReader, ErrorKind, Read, Write},
    path::PathBuf,
    sync::mpsc,
    thread,
    time::Duration,
};

use env_logger::Builder;
use jaypty::{Options, PseudoTerminalIO, PtySize, event::EventKind};
use jaysync::io::{
    ReadEventCapture, WriteEventCapture, WriteEvents,
    nonblocking::{NonBlockingPipeReader, NonBlockingPipeWriter},
};
use ptywin::contpy::ContpySpawn;

pub fn main() {
    let mut b = Builder::new();
    b.filter_level(log::LevelFilter::Info);
    b.init();

    let mut path = PathBuf::new();
    path.push("C:\\");
    let settings = Options {
        dimension: PtySize {
            columns: 24,
            rows: 80,
        },
        ..Default::default()
    };
    let mut io = ContpySpawn::new(settings);

    let mut cin = NonBlockingPipeWriter::new(io.take_cin(), 1024);

    thread::spawn(move || {
        let pipe = File::open("RELAY").unwrap();
        let mut reader = BufReader::new(pipe);
        loop {
            thread::sleep(Duration::from_millis(300));
            let mut buff = [0u8; 512];
            loop {
                match reader.read(&mut buff) {
                    Ok(0) => break,
                    Ok(count) => {
                        let content = String::from_utf8_lossy(&buff[..count]);
                        cin.write(content.as_bytes()).unwrap();
                    }
                    Err(e) => {
                        panic!("found error when reading {}", e);
                    }
                }
            }
        }
    });

    let (tx, rx) = mpsc::channel();
    let captured_cout: ReadEventCapture<miow::pipe::AnonRead, EventKind> =
        ReadEventCapture::new(io.take_cout(), tx.clone(), EventKind::CoutRead);
    let mut cout = NonBlockingPipeReader::new(captured_cout, 1024);
    loop {
        let rec: EventKind = rx.recv().unwrap();
        if rec == EventKind::CoutRead {
            let mut buff = [0u8; 512];
            loop {
                match cout.read(&mut buff) {
                    Ok(0) => break,
                    Ok(count) => {
                        let content = String::from_utf8_lossy(&buff[..count]);
                        print!("{}", content);
                        std::io::stdout().flush().unwrap();
                    }
                    Err(e) => {
                        panic!("found error {}", e);
                    }
                }
            }
        }
    }
}
