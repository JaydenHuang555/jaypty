use std::{fmt::Display, io, result};

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
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    #[error("unable to load internal")]
    LoadErrKind,
    #[error("active mspc channel got disconnected")]
    MspcChannelDisconnect,
    #[error("{0}")]
    ChildErr(ChildError),
    #[error("a data lock was poisoned")]
    PosionedLock,
}

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Self::new(value, None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Error {
    ctx: Option<String>,
    kind: Option<ErrorKind>,
}

pub type Result<T> = result::Result<T, Error>;

impl Error {
    pub fn new(kind: ErrorKind, ctx: Option<String>) -> Self {
        Self {
            ctx: ctx,
            kind: Some(kind),
        }
    }

    pub fn kind(kind: impl Into<ErrorKind>) -> Self {
        Self {
            kind: Some(kind.into()),
            ..Default::default()
        }
    }

    pub fn context<Input: Into<String>>(mut self, context: impl Into<Option<Input>>) -> Self {
        self.ctx = context.into().map(|ctx| ctx.into());
        self
    }

    pub fn append_os_error(mut self) -> Self {
        self.ctx = Some(self.ctx.map_or_else(
            || io::Error::last_os_error().to_string(),
            |mut ctx| {
                ctx.push_str(io::Error::last_os_error().to_string().as_str());
                ctx
            },
        ));
        self
    }
}

impl<T> Into<Result<T>> for Error {
    fn into(self) -> Result<T> {
        Err(self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_message = self
            .kind
            .clone()
            .map(|k| k.to_string())
            .unwrap_or("unknown error".to_string());
        if let Some(message) = self.ctx.as_ref() {
            write!(f, "(message: {}), kind error: {}", message, kind_message)
        } else {
            write!(f, "kind error: {}", kind_message)
        }
    }
}
