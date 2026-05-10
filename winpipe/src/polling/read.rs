use std::{
    io::{ErrorKind, Read},
    marker::PhantomData,
    sync::Arc,
    task::{Context, Poll, Waker},
    thread::{self, JoinHandle},
};

use jaysync::wake::ThreadWaker;
use piper::{Reader, pipe};
use polling::{
    Event, PollMode, Poller,
    os::iocp::{CompletionPacket, PollerIocpExt},
};

use crate::polling::{Polled, RegisteredPoll};

/// An asynchronous reader that writes the source
/// to an internal buffer on a seperate thread
pub struct PollingWakingNonBlockingPipeReader<R: Read> {
    /// This is the pipe reader.
    /// The pipe is the buffer that holds the actual read data
    /// and is updated on a seperate thread
    pipe: Reader, // read pipe
    handle: Option<JoinHandle<()>>, // flushing thread handle
    register: Arc<RegisteredPoll>,
    has_registered: bool,
    /// take ownership of the source pipe
    /// in order to avoid the source
    /// dropping while the thread
    /// is still active
    _src: PhantomData<R>,
}

impl<R: Read + Sized + Send + 'static> PollingWakingNonBlockingPipeReader<R> {
    pub fn new(mut source: R, capicity: usize) -> Self {
        // create async r/w memory

        // pipe is the reader
        //
        let (pipe, mut writer) = pipe(capicity);

        let h = thread::spawn(move || {
            let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
            let mut ctx = Context::from_waker(&waker);
            loop {
                // fill the pipe up with the bytes from the source
                match writer.poll_fill(&mut ctx, &mut source) {
                    // <If the pipe is full, this method returns Poll::Pending>
                    // park the thread until we are able to write to the pipe again
                    Poll::Pending => {
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
                    // else stop the thread
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
            handle: Some(h),
            register: Arc::new(RegisteredPoll::default()),
            has_registered: false,
            _src: PhantomData,
        }
    }

    pub fn register(&mut self, poller: &Arc<Poller>, event: Event, mode: Option<PollMode>) {
        if !event.readable {
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

    pub fn take_thread_handle(&mut self) -> Option<JoinHandle<()>> {
        self.handle.take()
    }
}

impl<R: Read> Read for PollingWakingNonBlockingPipeReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let waker = Waker::from(self.register.clone());
        let mut ctx = Context::from_waker(&waker);
        match self.pipe.poll_drain(&mut ctx, buf) {
            Poll::Pending => Ok(0),
            Poll::Ready(output) => output,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        thread,
        time::Duration,
    };

    use crate::polling::PollingWakingNonBlockingPipeReader;

    #[test]
    fn read() {
        const CAPICITY: usize = 1024;
        const PAYLOAD: &'static [u8] = b"PAYLOAD\n";
        const SLEEP_DURATION: Duration = Duration::from_millis(200);

        let (pipe_reader, mut pipe_writer) = miow::pipe::anonymous(1024).unwrap();
        let mut drain = PollingWakingNonBlockingPipeReader::new(pipe_reader, CAPICITY);
        pipe_writer.write(PAYLOAD).ok();
        thread::sleep(SLEEP_DURATION);
        let mut buff = [0u8; PAYLOAD.len()];
        let operation = drain.read(&mut buff);

        assert_eq!(PAYLOAD.len(), *operation.as_ref().unwrap());
        assert_eq!(&buff[..operation.unwrap()], PAYLOAD)
    }
}
