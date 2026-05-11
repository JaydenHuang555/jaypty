use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum PollRegisteringErrorKind {
    #[error("registering")]
    Register,
    #[error("reregistering")]
    ReRegister,
    #[error("deregistering")]
    DeRegister,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum FactoriedErrorKind {
    #[error("Failed to create Pty")]
    FailedPtyCreation,
    #[error("Failed pty resize")]
    FailedPtyResize,
    #[error("failed to latch child watch dog")]
    ChildWatchDogLatchFailed,
    #[error("failed to kill child")]
    KillChildFailed,
    #[error("encountered error when {0} intrest")]
    PollRegisteringFailed(#[from] PollRegisteringErrorKind),

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
