#[derive(Clone, Debug, Error)]
pub enum UnixPtyIOError {}

pub use UnixPtyIOError as SystemError;
