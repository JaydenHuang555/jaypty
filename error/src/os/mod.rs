#[cfg(windows)]
pub mod win;

#[cfg(windows)]
pub use win::*;

#[cfg(unix)]
pub mod unix;

#[cfg(unix)]
pub use unix::*;
