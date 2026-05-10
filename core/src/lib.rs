pub mod child;
pub mod command;
pub mod error;
pub mod io;
mod os_imports;
pub mod tokens;

pub use os_imports::*;

use std::path::PathBuf;

pub use io::PseudoTerminalIO;

use crate::command::Command;
pub use crate::error::Result;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub dimension: PtySize,
    pub cwd: Option<PathBuf>,
    pub cmd: Option<Command>,
}

impl Options {
    pub const fn default_const() -> Self {
        Self {
            dimension: PtySize {
                columns: 24,
                rows: 80,
            },
            cmd: None,
            cwd: None,
        }
    }

    pub const fn new(dimension: PtySize, cwd: Option<PathBuf>, cmd: Option<Command>) -> Self {
        Self {
            dimension,
            cmd,
            cwd,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            dimension: PtySize {
                columns: 24,
                rows: 80,
            },
            cwd: None,
            cmd: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PtySize {
    pub columns: usize,
    pub rows: usize,
}

impl PtySize {
    pub fn new(columns: usize, rows: usize) -> Self {
        Self { columns, rows }
    }
}

impl Default for PtySize {
    fn default() -> Self {
        Self {
            columns: 24,
            rows: 80,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn it_works() {}
}
