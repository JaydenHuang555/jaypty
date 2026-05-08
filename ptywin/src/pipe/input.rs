use std::{
    io::Write,
    sync::{Arc, Mutex, mpsc::Sender},
    task::{Context, Poll, Waker},
    thread::{self, JoinHandle},
};

use jaypty::{event::EventCaptureSource, pipe::PipeKind};
use jaysync::capture::HookableSource;
use piper::{Writer, pipe};
use polling::{
    Event, PollMode, Poller,
    os::iocp::{CompletionPacket, PollerIocpExt},
};

use crate::pipe::{ScheduledEvent, Task, ThreadWaker, output::NonBlockingPipeReader};

pub struct NonBlockingPipeWriter {
    pipe: Writer,
    handle: Option<JoinHandle<()>>,
}

// impl Drop for NonBlockingPipeWriter {
//     fn drop(&mut self) {
//         if let Some(handle) = self.handle.take() {
//             handle.join().unwrap();
//         }
//     }
// }

impl NonBlockingPipeWriter {
    pub fn new<Sink: 'static + Write + Send>(mut sink: Sink, pipe_capicity: usize) -> Self {
        let (mut reader, writer) = pipe(pipe_capicity);
        thread::spawn(move || {
            let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
            let mut cx = Context::from_waker(&waker);
            loop {
                match reader.poll_drain(&mut cx, &mut sink) {
                    std::task::Poll::Pending => {
                        thread::park();
                    }
                    Poll::Ready(Ok(0)) => {
                        log::info!("stopping writer");
                        return;
                    }
                    Poll::Ready(Ok(_)) => {
                        continue;
                    }
                    Poll::Ready(Err(e)) if e.kind() == std::io::ErrorKind::Interrupted => {}
                    Poll::Ready(Err(e)) => {
                        log::error!("found err {}", e);
                    }
                }
            }
        });
        Self {
            pipe: writer,
            handle: None,
        }
    }
}

impl Write for NonBlockingPipeWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(self.pipe.try_fill(buf))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
