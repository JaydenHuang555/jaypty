mod factory;

use std::result;

#[cfg(feature = "factory")]
pub use factory::*;

mod os;

pub use os::SystemError;

pub type Error = crate::factory::FactoriedError<SystemError>;

pub type Result<T> = std::result::Result<T, Error>;

pub type EmptyResult = Result<()>;

pub type OsError = SystemError;
pub type OsResult<T> = result::Result<T, SystemError>;
pub type OsEmptyResult = OsResult<()>;

impl<T> Into<OsResult<T>> for OsError {
    fn into(self) -> OsResult<T> {
        Err(self)
    }
}
