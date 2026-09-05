use core::fmt;
use std::str::FromStr;

use uuid::Uuid;

use crate::{error::AccountError, types::Platform};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountId(Uuid);
impl AccountId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}
impl FromStr for AccountId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value).map(Self)
    }
}
impl fmt::Display for AccountId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug)]
pub struct Account {
    id: AccountId,
    name: String,
    platform: Platform,
}

impl Account {
    pub(super) fn new(id: AccountId, name: String, platform: Platform) -> Self {
        Account { id, name, platform }
    }

    // Getters
    pub fn id(&self) -> &AccountId {
        &self.id
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn platform(&self) -> &Platform {
        &self.platform
    }

    pub(super) fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub(crate) fn validate_name(name: &str) -> Result<(), AccountError> {
        if name.trim().is_empty() {
            return Err(AccountError::InvalidName);
        }

        Ok(())
    }
}

pub struct Dataset {
    account_id: AccountId,
    dataset_type: String,
}

impl Dataset {
    pub(super) fn new(account_id: AccountId, dataset_type: String) -> Self {
        Dataset {
            account_id,
            dataset_type,
        }
    }

    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }
    pub fn dataset_type(&self) -> &String {
        &self.dataset_type
    }
}
