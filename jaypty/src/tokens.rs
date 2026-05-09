use polling::Event;

pub const TOKEN_READ: usize = 0;
pub const TOKEN_WRITE: usize = 1;
pub const TOKEN_CHILD_WATCH_DOG: usize = 2;

#[repr(usize)]
pub enum Token {
    CinRead = TOKEN_READ,
    CoutWrite = TOKEN_WRITE,
    ChildWatchDog = TOKEN_CHILD_WATCH_DOG,
}

impl Token {
    #[inline]
    pub const fn value(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn keyify(self, mut intrest: Event) -> Event {
        intrest.key = self as usize;
        intrest
    }
}

impl Into<Event> for Token {
    fn into(self) -> Event {
        match self {
            Self::CinRead => Event::readable(TOKEN_READ),
            Self::CoutWrite => Event::writable(TOKEN_WRITE),
            Self::ChildWatchDog => Event::readable(TOKEN_CHILD_WATCH_DOG),
        }
    }
}

impl Into<usize> for Token {
    fn into(self) -> usize {
        self as usize
    }
}
