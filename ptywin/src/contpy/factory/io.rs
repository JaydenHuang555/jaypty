use std::{
    io::{Read, Write},
    sync::mpsc::Sender,
};

use jaysync::io::{
    ReadEventCapture,
    nonblocking::{NonBlockingPipeReader, NonBlockingPipeWriter},
};

const PIPE_CAPTICITY: usize = 1024;

#[inline]
pub(crate) fn cout<Reader: Read + Send + 'static, Event: Clone + Send + 'static>(
    cout_source: Reader,
    tx: Sender<Event>,
    read_event: Event,
) -> NonBlockingPipeReader {
    let capture = ReadEventCapture::new(cout_source, tx, read_event);
    NonBlockingPipeReader::new(capture, PIPE_CAPTICITY)
}

#[inline]
pub(crate) fn cin<Writer: Write + Send + 'static>(cin_source: Writer) -> NonBlockingPipeWriter {
    NonBlockingPipeWriter::new(cin_source, PIPE_CAPTICITY)
}
