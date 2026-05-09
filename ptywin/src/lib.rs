pub mod child;
pub mod contpy;
#[cfg(not(windows))]
compile_error!("PLEASE COMPILE ON WINDOWS!");

#[cfg(test)]
mod tests {
    use std::{
        io::Write,
        path::PathBuf,
        sync::mpsc::{self, Receiver, Sender},
        thread,
        time::Duration,
    };

    use jaypty::{Options, PseudoTerminalIO, PtySize, event::EventKind};
    use jaysync::io::{
        ReadEventCapture, WriteEventCapture, WriteEvents,
        nonblocking::{NonBlockingPipeReader, NonBlockingPipeWriter},
    };

    use crate::contpy::ContpySpawn;

    #[test]
    fn blocking() {
        let mut path = PathBuf::new();
        path.push("C:\\");
        let settings = Options {
            dimension: PtySize {
                columns: 24,
                rows: 80,
            },
            cwd: Some(path),
            ..Default::default()
        };
        let mut io = ContpySpawn::spawn(settings);
        let (tx, rx): (Sender<EventKind>, Receiver<EventKind>) = mpsc::channel();
        let cin_capture = WriteEventCapture::new(
            io.take_cin(),
            tx.clone(),
            WriteEvents {
                write_event: Some(EventKind::CinWrite),
                ..Default::default()
            },
        );
        let mut cin = NonBlockingPipeWriter::new(cin_capture, 1024);
        let mut _cout = NonBlockingPipeReader::new(
            ReadEventCapture::new(io.take_cout(), tx.clone(), EventKind::CoutRead),
            1024,
        );

        cin.write(String::from_utf8_lossy(b"dir\n").as_bytes())
            .unwrap();
        thread::sleep(Duration::from_millis(300));
        assert_eq!(
            {
                let unwrapped = rx.try_recv().unwrap();
                unwrapped == EventKind::CinWrite || unwrapped == EventKind::CoutRead
            },
            true
        )
    }
}
