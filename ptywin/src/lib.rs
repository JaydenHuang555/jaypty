pub mod child;
pub mod poll;

pub mod error;
pub mod factory;
pub mod io;
pub mod symbols;

pub(crate) use symbols::ContpyHandle;
pub(crate) use symbols::ContpySymbols;
pub(crate) use symbols::loaded_symbols;

#[cfg(not(windows))]
compile_error!("PLEASE COMPILE ON WINDOWS!");

pub use crate::error::Error;
pub use crate::error::Result;
pub use poll::RegisteredPoll;

#[cfg(test)]
mod tests {

    use jaysync::io::{
        ReadEventCapture, WriteEventCapture, WriteEvents,
        nonblocking::{NonBlockingPipeReader, NonBlockingPipeWriter},
    };
}
