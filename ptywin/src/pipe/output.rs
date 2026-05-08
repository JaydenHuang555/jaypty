use std::{
    io::{ErrorKind, Read},
    marker::PhantomData,
    sync::{Arc, Mutex, mpsc::Sender},
    task::{Context, Poll, Wake, Waker},
    thread::{self, JoinHandle},
};

use jaypty::{event::EventKind, pipe::PipeKind};
use miow::pipe::{AnonRead, AnonWrite};
use piper::{Reader, pipe};
use polling::{
    Event, Events, PollMode, Poller,
    os::iocp::{CompletionPacket, PollerIocpExt},
};

use crate::pipe::{ScheduledEvent, Task, ThreadWaker};

pub struct NonBlockingPipeReader {
    pipe: Reader,
    join_handle: Option<JoinHandle<()>>,
}

// impl Drop for NonBlockingPipeReader {
//     fn drop(&mut self) {
//         if let Some(handle) = self.join_handle.take() {
//             handle.join().unwrap();
//         }
//     }
// }

impl NonBlockingPipeReader {
    pub fn new<R: Read + Sized + Send + 'static>(mut source: R, capicity: usize) -> Self {
        let (pipe, mut writer) = pipe(capicity);

        let h = thread::spawn(move || {
            let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
            let mut ctx = Context::from_waker(&waker);
            loop {
                match writer.poll_fill(&mut ctx, &mut source) {
                    Poll::Pending => {
                        log::info!("reader pending");
                        thread::park();
                    }
                    Poll::Ready(Ok(0)) => {
                        log::info!("returning");
                        return;
                    }
                    Poll::Ready(Ok(c)) => {
                        log::info!("read {} bytes", c);
                        continue;
                    }
                    Poll::Ready(Err(e)) => {
                        log::info!("found error");
                        if e.kind() == ErrorKind::Interrupted {
                            continue;
                        } else {
                            panic!("error when writing output {}", e);
                        }
                    }
                }
            }
        });

        Self {
            pipe,
            join_handle: None,
        }
    }
}

impl Read for NonBlockingPipeReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        log::info!("read");

        Ok(self.pipe.try_drain(buf))
    }
}
