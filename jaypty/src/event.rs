use jaysync::capture::HookableSource;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum EventKind {
    CinWrite,
    CoutRead,
}

pub trait EventCaptureSource: HookableSource<EventKind> {}
