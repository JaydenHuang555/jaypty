use std::env::var;

use thiserror::Error;

const USER_ENV_KEY: &'static str = "USER";
const HOME_ENV_KEY: &'static str = "HOME";
const SHELL_ENV_KEY: &'static str = "SHELL";

#[derive(Error, Debug, Clone)]
pub enum AccountGetterError {
    #[error("unable to get env var for {0}. var error is :{1}")]
    Env(&'static str, std::env::VarError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub username: String,
    pub home: String,
    pub shell: String,
}

impl Account {
    pub fn env_account() -> Result<Account, AccountGetterError> {
        Ok(Self {
            home: var(HOME_ENV_KEY).map_err(|e| AccountGetterError::Env(HOME_ENV_KEY, e))?,
            username: var(USER_ENV_KEY).map_err(|e| AccountGetterError::Env(USER_ENV_KEY, e))?,
            shell: var(SHELL_ENV_KEY).map_err(|e| AccountGetterError::Env(SHELL_ENV_KEY, e))?,
        })
    }
}
