use std::{
    io::{ErrorKind, Read, Write},
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
    thread::{self, JoinHandle},
};

use crate::wake::ThreadWaker;
use piper::{Reader, Writer, pipe};

/// nonblocking.rs but with support for polling

/// An asynchronous writer that will write to a sink on a seperate thread.
/// On a write operation, the bytes are copied to an async r/w internal buffer
pub struct WakingNonBlockingPipeWriter<W: Wake + Send + Sync> {
    /// This is the writer to the pipe
    /// The pipe is the async r/w buffer where info gets copied into
    /// during the write calls
    pipe: Writer,
    /// copying thread
    /// the copying thread is the thread
    /// where the internal buffer drain's it contents
    /// into the sink
    handle: Option<JoinHandle<()>>,

    wake: Arc<W>,
}

impl<W: Wake + Send + Sync> WakingNonBlockingPipeWriter<W> {
    pub fn new<Sink: 'static + Write + Send + 'static>(
        mut sink: Sink,
        pipe_capicity: usize,
        wake: W,
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
            wake: Arc::new(wake),
        }
    }

    pub fn take_thread_handle(&mut self) -> Option<JoinHandle<()>> {
        self.handle.take()
    }

    pub fn map_wake(&mut self, function: impl Fn(&mut Arc<W>)) {
        (function)(&mut self.wake)
    }

    pub fn wake_mut(&mut self) -> &mut Arc<W> {
        &mut self.wake
    }
}

impl<W: Wake + Send + Sync + 'static> Write for WakingNonBlockingPipeWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let waker = Waker::from(self.wake.clone());
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

/// An asynchronous reader that writes the source
/// to an internal buffer on a seperate thread
pub struct WakingNonBlockingPipeReader<W: Wake + Sync + Send + 'static> {
    /// This is the pipe reader.
    /// The pipe is the buffer that holds the actual read data
    /// and is updated on a seperate thread
    pipe: Reader, // read pipe
    handle: Option<JoinHandle<()>>, // flushing thread handle
    wake: Arc<W>,
}

impl<W: Wake + Send + Sync> WakingNonBlockingPipeReader<W> {
    pub fn new<R: Read + Sized + Send + 'static>(mut source: R, capicity: usize, wake: W) -> Self {
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
            wake: Arc::new(wake),
        }
    }

    pub fn take_thread_handle(&mut self) -> Option<JoinHandle<()>> {
        self.handle.take()
    }

    pub fn map_wake(&mut self, function: impl Fn(&mut Arc<W>)) {
        (function)(&mut self.wake)
    }

    pub fn wake_mut(&mut self) -> &mut Arc<W> {
        &mut self.wake
    }
}

impl<W: Wake + Sync + Send + 'static> Read for WakingNonBlockingPipeReader<W> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let waker = Waker::from(self.wake.clone());
        let mut ctx = Context::from_waker(&waker);
        match self.pipe.poll_drain(&mut ctx, buf) {
            Poll::Pending => Ok(0),
            Poll::Ready(output) => output,
        }
    }
}
