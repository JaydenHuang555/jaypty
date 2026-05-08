pub trait PtyReadPipe {
    fn read(&self);
}

pub trait PtyWritePipe {
    fn write(&self);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipeKind {
    Write,
    Read,
}

impl PipeKind {
    pub fn is_readible(&self) -> bool {
        match self {
            Self::Read => true,
            Self::Write => false,
        }
    }

    pub fn is_writable(&self) -> bool {
        match self {
            Self::Read => false,
            Self::Write => true,
        }
    }
}
