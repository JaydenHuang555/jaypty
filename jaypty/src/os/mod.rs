#[cfg(any(windows, feature = "win"))]
mod win;

#[cfg(any(windows, feature = "win"))]
pub use win::*;

#[cfg(any(unix, feature = "unix"))]
mod unix;

#[cfg(any(unix, feature = "unix"))]
pub use unix::*;
