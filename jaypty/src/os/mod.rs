#[cfg(any(windows, feature = "win"))]
mod win;

#[cfg(any(windows, feature = "win"))]
pub use win::*;
