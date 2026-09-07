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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetType {
    TwitterDirectMessages,
    Unknown,
}
impl DatasetType {
    pub fn from_file_name(filename: String) -> Self {
        match filename.as_str() {
            "direct-messages.js" => Self::TwitterDirectMessages,
            _ => Self::Unknown,
        }
    }

    pub fn platform(self) -> Platform {
        match self {
            Self::TwitterDirectMessages => Platform::Twitter,
            Self::Unknown => Platform::Unknown,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::TwitterDirectMessages => "twitter.direct_messages",
            Self::Unknown => "unknown",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::TwitterDirectMessages => "Direct Messages",
            Self::Unknown => "Unknown",
        }
    }

    pub fn file_name(self) -> &'static str {
        match self {
            Self::TwitterDirectMessages => "direct-messages.js",
            Self::Unknown => "unknown",
        }
    }
}

pub struct Dataset {
    account_id: AccountId,
    dataset_type: DatasetType,
}

impl Dataset {
    pub(super) fn new(account_id: AccountId, dataset_type: DatasetType) -> Self {
        Dataset {
            account_id,
            dataset_type,
        }
    }

    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }
    pub fn dataset_type(&self) -> &DatasetType {
        &self.dataset_type
    }
}
