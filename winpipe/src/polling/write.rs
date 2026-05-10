use std::{
    io::Write,
    sync::Arc,
    task::{Context, Poll, Waker},
    thread::{self, JoinHandle},
};

use jaysync::wake::ThreadWaker;
use piper::{Writer, pipe};
use polling::{
    Event, PollMode, Poller,
    os::iocp::{CompletionPacket, PollerIocpExt},
};

use crate::polling::{Polled, RegisteredPoll};

/// nonblocking.rs but with support for polling

/// An asynchronous writer that will write to a sink on a seperate thread.
/// On a write operation, the bytes are copied to an async r/w internal buffer
pub struct PollingWakingNonBlockingPipeWriter {
    /// This is the writer to the pipe
    /// The pipe is the async r/w buffer where info gets copied into
    /// during the write calls
    pipe: Writer,
    /// copying thread
    /// the copying thread is the thread
    /// where the internal buffer drain's it contents
    /// into the sink
    handle: Option<JoinHandle<()>>,

    register: Arc<RegisteredPoll>,
    has_registered: bool,
}

impl PollingWakingNonBlockingPipeWriter {
    pub fn new<Sink: 'static + Write + Send + 'static>(
        mut sink: Sink,
        pipe_capicity: usize,
    ) -> Self {
        let (mut reader, writer) = pipe(pipe_capicity);
        let j = thread::spawn(move || {
            let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
            let mut cx = Context::from_waker(&waker);
            loop {
                // drain the buffer into the sink
                match reader.poll_drain(&mut cx, &mut sink) {
                    // <If the pipe is full, this method returns Poll::Pending>
                    // park the thread until we are able to write to the pipe again
                    std::task::Poll::Pending => {
                        thread::park();
                    }
                    // <If the pipe is closed, this method returns Poll::Ready(Ok(0))>
                    // We are unable to use this pipe
                    Poll::Ready(Ok(0)) => {
                        return;
                    }
                    //<Otherwise, this method returns Poll::Ready(Ok(n)) where n is the number of bytes read.>
                    // just move along
                    Poll::Ready(Ok(_)) => {
                        continue;
                    }
                    //<Errors in src are bubbled up through Poll::Ready(Err(e))>
                    // if it is interrupted, it is fine to continue reading
                    Poll::Ready(Err(e)) if e.kind() == std::io::ErrorKind::Interrupted => {}
                    // else stop the thread
                    Poll::Ready(Err(e)) => {
                        panic!("found error {}", e)
                    }
                }
            }
        });
        Self {
            pipe: writer,
            handle: Some(j),
            register: Arc::new(RegisteredPoll::default()),
            has_registered: false,
        }
    }

    pub fn take_thread_handle(&mut self) -> Option<JoinHandle<()>> {
        self.handle.take()
    }

    pub fn register(&mut self, poller: &Arc<Poller>, event: Event, mode: Option<PollMode>) {
        if !event.writable {
            self.unregister();
            return;
        }
        {
            let mut lock = self.register.polled.lock().unwrap();
            *lock = Some(Polled::new(poller, event, mode));
            drop(lock);
        }
        if !self.pipe.is_empty() || !self.has_registered {
            self.has_registered = true;
            poller.post(CompletionPacket::new(event)).unwrap();
        }
    }

    pub fn unregister(&mut self) {
        let mut lock = self.register.polled.lock().unwrap();
        *lock = None;
    }
}

impl Write for PollingWakingNonBlockingPipeWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let waker = Waker::from(self.register.clone());
        let mut ctx = Context::from_waker(&waker);
        match self.pipe.poll_fill(&mut ctx, buf) {
            Poll::Pending => Ok(0),
            Poll::Ready(output) => output,
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
