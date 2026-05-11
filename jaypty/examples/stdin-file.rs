use jaypty::ChildWatchDogIO;
use jaypty::PollIntrest;
use jaypty::PolledEvents;
use std::{
    fs::File,
    io::{BufReader, ErrorKind, Read, Write},
    sync::{Arc, Mutex},
    thread,
};

use jaypty::{
    Options, Poller, PseudoTerminalIO, SystemPseudoTerminalIO, PollingIntrestRegisterIO,
};

const RELAY_FNAME: &'static str = "RELAY";

// TODO: add compile support for unix
#[cfg(windows)]
fn stdin_relay() {
    File::create(RELAY_FNAME).unwrap();
    let io = Arc::new(Mutex::new(SystemPseudoTerminalIO::new(Options::default())));
    let mut watch_dog = {
        let lock = io.lock().unwrap();
        lock.spawn_and_latch_child_watchdog()
    };
    let poller = Arc::new(Poller::new().unwrap());
    let current_event = PollIntrest::readable(0);
    unsafe {
        io.lock().unwrap().register(&poller, current_event, None);
    }

    let io_writer = Arc::clone(&io);
    thread::spawn(move || {
        let mut reader = BufReader::new(File::open(RELAY_FNAME).unwrap());
        loop {
            let mut buff = [0u8; 512];
            loop {
                match reader.read(&mut buff) {
                    Ok(n) => {
                        let mut lock = io_writer.lock().unwrap();
                        lock.write(&buff[..n]).expect("unable to write to cout");
                    }
                    Err(n) if n.kind() == ErrorKind::Interrupted => {}
                    Err(n) => {
                        panic!("found error when reading from file {}", n);
                    }
                }
            }
        }
    });

    let io_reader = Arc::clone(&io);
    let read_poller = Arc::clone(&poller);
    thread::spawn(move || {
        let mut events = PolledEvents::new();
        loop {
            events.clear();
            read_poller
                .wait(&mut events, None)
                .expect("found polling error");
            let mut buff = [0u8; 512];

            for event in events.iter() {
                if !event.readable {
                    return;
                }
                loop {
                    let operation = {
                        let mut lock = io_reader.lock().unwrap();
                        lock.read(&mut buff)
                    };
                    match operation {
                        Ok(n) => {
                            if n == 0 {
                                break;
                            }
                            let content = String::from_utf8_lossy(&buff[..n]);
                            print!("{}", content);
                            std::io::stdout().flush().unwrap();
                        }
                        Err(e) => {
                            if e.kind() != ErrorKind::Interrupted {
                                panic!("found error when reading from cout");
                            }
                        }
                    }
                }
                unsafe {
                    io.lock()
                        .unwrap()
                        .reregister(&read_poller, PollIntrest::readable(0), None);
                }
            }
        }
    });
    watch_dog.wait();
}

pub fn main() {
    #[cfg(windows)]
    stdin_relay()
}
