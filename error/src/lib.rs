mod factory;

#[cfg(feature = "factory")]
pub use factory::*;

mod os;

pub use os::*;

pub type Error = crate::factory::FactoriedError<SystemError>;

pub type Result<T> = std::result::Result<T, Error>;
