use uuid::Uuid;

use crate::{error::AccountError, types::Platform};

#[derive(Debug)]
pub struct Account {
    id: Uuid,
    name: String,
    platform: Platform,
}

impl Account {
    /// To create an account instance to be passed to the repository, name and platform must be set. uuid is
    /// automatically generated.
    ///
    /// ```
    /// use kolib::export_reader::account::Account;
    /// use kolib::types::Platform;
    /// let account = Account::new().name("@my_old_acc".to_string()).platform(Platform::Twitter);
    /// ```
    pub(super) fn new(id: Uuid, name: String, platform: Platform) -> Self {
        Account {
            id: id,
            name: name,
            platform: platform,
        }
    }

    // Getters
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn platform(&self) -> &Platform {
        &self.platform
    }

    pub(crate) fn validate_name(name: &str) -> Result<(), AccountError> {
        if name.trim().is_empty() {
            return Err(AccountError::InvalidName);
        }

        Ok(())
    }
}

pub struct Dataset {
    account_id: Uuid,
    dataset_type: String,
}

impl Dataset {
    pub(super) fn new(account_id: Uuid, dataset_type: String) -> Self {
        Dataset {
            account_id,
            dataset_type,
        }
    }

    pub fn account_id(&self) -> Uuid {
        self.account_id
    }
    pub fn dataset_type(&self) -> &String {
        &self.dataset_type
    }
}
