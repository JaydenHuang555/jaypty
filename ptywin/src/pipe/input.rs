use std::{
    io::Write,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread::{self, JoinHandle},
};

use jaypty::pipe::PipeKind;
use piper::{Writer, pipe};
use polling::{Event, PollMode, Poller};

use crate::pipe::{RegisteredTask, ThreadWaker, WrappedRegisteredTask};

pub struct NonBlockingPipeWriter {
    pipe: Writer,
    handle: Option<JoinHandle<()>>,
    task: Arc<WrappedRegisteredTask>,
}

impl Drop for NonBlockingPipeWriter {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

impl NonBlockingPipeWriter {
    pub fn new<Sink: 'static + Write + Send>(mut sink: Sink, pipe_capicity: usize) -> Self {
        let (mut reader, writer) = pipe(pipe_capicity);
        let handle = thread::spawn(move || {
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
            handle: Some(handle),
            task: Arc::new(WrappedRegisteredTask {
                kind: PipeKind::Write,
                task: Mutex::new(None),
            }),
        }
    }

    pub fn register(&mut self, poller: &Arc<Poller>, event: Event, mode: PollMode) {
        let mut task = self.task.task.lock().unwrap();
        *task = Some(RegisteredTask {
            poller: poller.clone(),
            event: event,
            mode: mode,
        });
    }
}

impl Write for NonBlockingPipeWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let waker = Waker::from(self.task.clone());
        let mut ctx = Context::from_waker(&waker);
        match self.pipe.poll_fill_bytes(&mut ctx, buf) {
            Poll::Pending => Ok(0),
            Poll::Ready(stat) => Ok(stat),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
