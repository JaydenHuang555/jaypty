use std::{
    io::{ErrorKind, Read},
    marker::PhantomData,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
    thread::{self, JoinHandle},
};

use jaypty::{message::Message, pipe::PipeKind};
use miow::pipe::{AnonRead, AnonWrite};
use piper::{Reader, pipe};
use polling::{
    Event, Events, PollMode, Poller,
    os::iocp::{CompletionPacket, PollerIocpExt},
};

use crate::pipe::{RegisteredTask, ThreadWaker, WrappedRegisteredTask};

pub struct NonBlockingPipeReader {
    pipe: Reader,
    join_handle: Option<JoinHandle<()>>,
    tasks: Arc<WrappedRegisteredTask>,
}

impl Drop for NonBlockingPipeReader {
    fn drop(&mut self) {
        if let Some(handle) = self.join_handle.take() {
            handle.join().unwrap();
        }
    }
}

impl NonBlockingPipeReader {
    pub fn new<R: Read + Sized + Send + 'static>(mut source: R, capicity: usize) -> Self {
        let (pipe, mut writer) = pipe(capicity);

        let h = thread::spawn(move || {
            let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
            let mut ctx = Context::from_waker(&waker);
            loop {
                println!("reading");
                match writer.poll_fill(&mut ctx, &mut source) {
                    Poll::Pending => {
                        thread::park();
                    }
                    Poll::Ready(Ok(0)) => {
                        return;
                    }
                    Poll::Ready(Ok(c)) => {
                        println!("read {} bytes", c);
                        continue;
                    }
                    Poll::Ready(Err(e)) => {
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
            join_handle: Some(h),
            tasks: Arc::new(WrappedRegisteredTask::new(PipeKind::Read)),
        }
    }

    pub fn register(&mut self, poller: &Arc<Poller>, event: Event, mode: PollMode) {
        let mut task = self.tasks.task.lock().unwrap();
        *task = Some(RegisteredTask {
            poller: poller.clone(),
            event: event,
            mode: mode,
        });
    }
}

impl Read for NonBlockingPipeReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let waker = Waker::from(self.tasks.clone());
        let mut ctx = Context::from_waker(&waker);

        match self.pipe.poll_drain_bytes(&mut ctx, buf) {
            Poll::Pending => Ok(0),
            Poll::Ready(result) => Ok(result),
        }
    }
}
