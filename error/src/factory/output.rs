use std::error::Error;

use super::FactoriedErrorKind;

#[derive(Debug, PartialEq, Eq)]
pub struct FactoriedError<SystemError: Error> {
    pub(super) kind: FactoriedErrorKind,
    pub(super) internal: Option<SystemError>,
    pub(super) errno: Option<i32>,
}

impl<SystemError: Error> FactoriedError<SystemError> {
    pub fn new<S: Into<Option<SystemError>>>(
        kind: FactoriedErrorKind,
        internal: S,
        errno: impl Into<Option<i32>>,
    ) -> Self {
        Self {
            kind,
            internal: internal.into(),
            errno: errno.into(),
        }
    }

    pub const fn new_const(
        kind: FactoriedErrorKind,
        internal: Option<SystemError>,
        errno: Option<i32>,
    ) -> Self {
        Self {
            kind,
            internal,
            errno,
        }
    }

    pub fn with_kind(mut self, kind: FactoriedErrorKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_internal(mut self, internal: impl Into<Option<SystemError>>) -> Self {
        self.internal = internal.into();
        self
    }

    pub fn with_errno(mut self, errno: impl Into<Option<i32>>) -> Self {
        self.errno = errno.into();
        self
    }

    pub fn errno(&self) -> Option<i32> {
        self.errno
    }

    pub fn kind(&self) -> FactoriedErrorKind {
        self.kind
    }

    pub fn internal(&self) -> Option<&SystemError> {
        self.internal.as_ref()
    }
}
