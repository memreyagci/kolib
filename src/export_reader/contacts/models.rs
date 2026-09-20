use core::fmt;
use std::str::FromStr;

use uuid::Uuid;

use crate::error::ContactError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContactId(Uuid);

impl ContactId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl FromStr for ContactId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value).map(Self)
    }
}

impl fmt::Display for ContactId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug)]
pub struct Contact {
    id: ContactId,
    name: String,
    is_me: bool,
}

impl Contact {
    pub(super) fn new(id: ContactId, name: String, is_me: bool) -> Self {
        Self { id, name, is_me }
    }

    pub fn id(&self) -> &ContactId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_me(&self) -> bool {
        self.is_me
    }

    pub(super) fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub(crate) fn validate_name(name: &str) -> Result<(), ContactError> {
        if name.trim().is_empty() {
            return Err(ContactError::InvalidName);
        }

        Ok(())
    }
}
