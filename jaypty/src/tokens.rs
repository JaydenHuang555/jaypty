use polling::Event;

bitflags::bitflags! {
    pub struct TokenFlags: u32 {
        const READABLE = 0;
        const WRITEABLE = 1;
    }
}

pub const TOKEN_READ: usize = 0;
pub const TOKEN_WRITE: usize = 1;
pub const TOKEN_CHILD_WATCH_DOG: usize = 2;

#[repr(usize)]
pub enum Token {
    CinWrite = TOKEN_WRITE,
    CoutRead = TOKEN_READ,
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

    #[inline]
    pub const fn intrest(self) -> Event {
        let flags = {
            match self {
                Self::CinWrite => TokenFlags::WRITEABLE,
                Self::CoutRead => TokenFlags::READABLE,
                Self::ChildWatchDog => TokenFlags::READABLE,
            }
        };
        let is_writable = flags.contains(TokenFlags::WRITEABLE);
        let is_readable = flags.contains(TokenFlags::READABLE);
        let key = self as usize;
        Event::new(key, is_readable, is_writable)
    }

    // pub const fn eventify(self) -> Event {
    //     Event::
    // }
}

impl Into<Event> for Token {
    fn into(self) -> Event {
        match self {
            Self::CinWrite => Event::readable(TOKEN_READ),
            Self::CoutRead => Event::writable(TOKEN_WRITE),
            Self::ChildWatchDog => Event::readable(TOKEN_CHILD_WATCH_DOG),
        }
    }
}

impl Into<usize> for Token {
    fn into(self) -> usize {
        self as usize
    }
}
