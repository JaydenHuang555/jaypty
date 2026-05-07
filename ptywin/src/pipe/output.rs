use std::{
    io::{ErrorKind, Read},
    marker::PhantomData,
    sync::Arc,
    task::{Context, Poll, Waker},
    thread::{self, JoinHandle},
};

use miow::pipe::{AnonRead, AnonWrite};
use piper::{Reader, pipe};

use crate::pipe::ThreadWaker;

pub struct NonBlockingPipeReader {
    pipe: Reader,
    join_handle: Option<JoinHandle<()>>,
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
                match writer.poll_fill(&mut ctx, &mut source) {
                    Poll::Pending => {
                        thread::park();
                    }
                    Poll::Ready(Ok(0)) => {
                        return;
                    }
                    Poll::Ready(Ok(_)) => {
                        continue;
                    }
                    Poll::Ready(Err(e)) => {
                        if e.kind() == ErrorKind::Interrupted {
                            continue;
                        } else {
                            log::error!("error when writing output {}", e);
                        }
                    }
                }
            }
        });

        Self {
            pipe,
            join_handle: Some(h),
        }
    }
}

impl Read for NonBlockingPipeReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.pipe.try_drain(buf);
        Ok(n)
    }
}
