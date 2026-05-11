use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum ContpyIOError {}

pub use ContpyIOError as SystemError;
