use std::{
    io::{ErrorKind, Read, Write},
    sync::Arc,
    task::{Context, Poll, Waker},
    thread::{self, JoinHandle},
};

use crate::wake::ThreadWaker;
use piper::{Reader, Writer, pipe};

pub struct NonBlockingPipeWriter {
    pipe: Writer,
    handle: Option<JoinHandle<()>>,
}

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
                        return;
                    }
                    Poll::Ready(Ok(_)) => {
                        continue;
                    }
                    Poll::Ready(Err(e)) if e.kind() == std::io::ErrorKind::Interrupted => {}
                    Poll::Ready(Err(e)) => {}
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
                        thread::park();
                    }
                    Poll::Ready(Ok(0)) => {
                        return;
                    }
                    Poll::Ready(Ok(c)) => {
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
            join_handle: None,
        }
    }
}

impl Read for NonBlockingPipeReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(self.pipe.try_drain(buf))
    }
}
