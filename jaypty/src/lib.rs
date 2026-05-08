pub mod event;
pub mod io;
pub mod pipe;
pub mod tokens;

pub use io::JayPtyIO;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
