use std::{collections::BTreeMap, ffi::OsString};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    name: String,
    args: Vec<String>,
}

impl Command {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            args: Vec::new(),
        }
    }

    pub const fn process_name(&self) -> &String {
        &self.name
    }

    pub const fn args(&self) -> &Vec<String> {
        &self.args
    }
}
