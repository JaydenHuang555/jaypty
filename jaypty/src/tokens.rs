use polling::Event;

use crate::pipe::PtyWritePipe;

pub const TOKEN_READ: usize = 0;
pub const TOKEN_WRITE: usize = 1;

#[repr(usize)]
pub enum PtyTokens {
    Read = TOKEN_READ,
    Write = TOKEN_WRITE,
}

impl PtyTokens {
    pub const fn value(self) -> usize {
        self as usize
    }
}

impl Into<Event> for PtyTokens {
    fn into(self) -> Event {
        match self {
            Self::Read => Event::readable(TOKEN_READ),
            Self::Write => Event::writable(TOKEN_WRITE),
        }
    }
}

impl Into<usize> for PtyTokens {
    fn into(self) -> usize {
        self as usize
    }
}
