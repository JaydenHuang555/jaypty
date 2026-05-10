use std::{fmt::Display, io, result};

use jaypty::error::ErrorKind;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LoadErrorKind {
    #[error("unable to find contpy.dll")]
    UnableToFindContpyDll,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ChildError {
    #[error("unable to aquire exit code ")]
    UnableToGetExitCode,
    #[error("child is alive when the exit callback watchdog was called")]
    AliveDuringExitCallBack,
}

impl Into<ErrorKind> for ChildError {
    fn into(self) -> ErrorKind {
        ErrorKind::OsSpecific(self.to_string())
    }
}
