use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum FactoriedErrorKind {
    #[error("Failed to create Pty")]
    FailedPtyCreation,

    /// THESE SHOULD NEVER BE PART OF A PR!!
    /// THESE ARE JUST FOR PLACE HOLDER VALUES
    /// FOR WHEN U JUST NEED A RETURN VALUE
    /// OR DEBUGGING SOMETHING.

    #[error("Place Holder Error with content: {0}")]
    PlaceHolderContext(&'static str),
    #[error("Place Holder Errorno: {0}")]
    PlaceHolderErrno(i32),
    #[error("Place Holder Err Context: ({0}) Errno: {1}")]
    PlaceHolderErrnoContext(&'static str, i32),
}
