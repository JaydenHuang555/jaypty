pub mod command;
pub mod event;
pub mod io;
pub mod pipe;
pub mod subsystem;
pub mod tokens;

use std::path::PathBuf;

pub use io::PseudoTerminalIO;

use crate::command::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub dimension: PtySize,
    pub cwd: Option<PathBuf>,
    pub cmd: Option<Command>,
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

pub fn add(left: u64, right: u64) -> u64 {
    left + right
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

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
