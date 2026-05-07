use std::{fmt::Display, result};

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LoadErrorKind {
    #[error("unable to find contpy.dll")]
    UnableToFindContpyDll,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    #[error("unable to load internal")]
    LoadErrKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: Option<String>,
    kind: ErrorKind,
}

pub type Result<T> = result::Result<T, Error>;

impl Error {
    pub fn new(message: impl Into<Option<String>>, kind: ErrorKind) -> Self {
        Self {
            message: message.into(),
            kind,
        }
    }

    pub fn load_err<Input: AsRef<str>>(message: impl Into<Option<Input>>) -> Self {
        Self {
            message: message.into().map(|s| s.as_ref().to_string()),
            kind: ErrorKind::LoadErrKind,
        }
    }
}

impl<T> Into<Result<T>> for Error {
    fn into(self) -> Result<T> {
        Err(self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_message = self.kind.to_string();
        if let Some(message) = self.message.as_ref() {
            write!(f, "(message: {}), kind error: {}", message, kind_message)
        } else {
            write!(f, "kind error: {}", kind_message)
        }
    }
}
