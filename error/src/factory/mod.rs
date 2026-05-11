use std::{error::Error, fmt::Display};

mod kind;
mod output;

pub use kind::*;
pub use output::FactoriedError;

impl<S: Error + Clone> Display for FactoriedError<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ found error (type: {}) with params of (internal: {:?}) (errno: {:?}) }}",
            self.kind, self.internal, self.errno
        )
    }
}

pub struct ErrorFactory {}

impl ErrorFactory {
    pub const fn kind_const<S: Error>(kind: FactoriedErrorKind) -> FactoriedError<S> {
        FactoriedError {
            kind,
            internal: None,
            errno: None,
        }
    }

    pub fn kind<S: Error>(kind: impl Into<FactoriedErrorKind>) -> FactoriedError<S> {
        FactoriedError {
            kind: kind.into(),
            internal: None,
            errno: None,
        }
    }

    pub fn place_holder_const<S: Error>(msg: &'static str) -> FactoriedError<S> {
        FactoriedError {
            kind: FactoriedErrorKind::PlaceHolderContext(msg),
            internal: None,
            errno: None,
        }
    }

    pub fn place_holder<S: Error>(msg: impl Into<&'static str>) -> FactoriedError<S> {
        FactoriedError {
            kind: FactoriedErrorKind::PlaceHolderContext(msg.into()),
            internal: None,
            errno: None,
        }
    }
}
