use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use jaypty::PtySize;
use jaypty::tokens::{PtyTokens, TOKEN_WRITE};
use polling::{Event, Events, PollMode, Poller};
use ptywin::contpy::ContpyIO;
use ptywin::pipe::RegisteredTask;
use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole, FreeConsole};

#[derive(Clone, Debug)]
pub struct State {
    built_content: String,
}

pub fn main() {
    let mut io = ContpyIO::new(PtySize {
        columns: 24,
        rows: 80,
    });
    let writer = io.writer().clone();
    let reader = io.reader().clone();
    let child_watch_dog = io.child_watch_dog().clone();
    let mut relay_writer = BufWriter::new(File::create("RELAY").unwrap());
    let (tx, rx) = std::sync::mpsc::channel();
    let log_handle = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            let mut buff = [0u8; 512];
            loop {
                let operation = {
                    let mut read = reader.write().unwrap();
                    let ret = read.read(&mut buff);
                    drop(read);
                    ret
                };
                if let Err(e) = operation {
                    eprintln!("found error {}", e);
                    panic!()
                }
                let n = operation.unwrap();
                if n == 0 {
                    break;
                } else if n > 0 {
                    relay_writer.write(&buff[..n]).unwrap();
                    relay_writer.flush().unwrap();
                }
            }
        }
    });
    let read_handle = thread::spawn(move || {
        loop {
            let mut content = [0u8; 512];
            println!("please enter command");

            let counted = std::io::stdin().read(&mut content).unwrap();

            if (&content[..counted]).to_ascii_lowercase() == b"exit" {
                break;
            }
            {
                let mut write = writer.write().unwrap();
                write.write(&content[..counted]).unwrap();
            }
        }
        tx.send(true).unwrap();
    });
    let rec = rx.recv();
    match rec {
        Ok(stat) => {
            println!("able to exit");
        }
        Err(e) => {
            eprintln!("rec v error: {}", e);
        }
    }
    read_handle.join().unwrap();
    log_handle.join().unwrap();
}
