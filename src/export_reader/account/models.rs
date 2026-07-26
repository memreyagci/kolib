use uuid::Uuid;

use crate::types::Platform;

#[derive(Debug)]
pub struct AccountModel {
    id: Uuid,
    name: String,
    platform: Platform,
}

impl AccountModel {
    /// To create an account instance to be passed to the repository, name and platform must be set. uuid is
    /// automatically generated.
    ///
    /// ```
    /// use kolib::export_reader::account::Account;
    /// use kolib::types::Platform;
    /// let account = Account::new().name("@my_old_acc".to_string()).platform(Platform::Twitter);
    /// ```
    pub fn new(name: String, platform: Platform) -> Self {
        AccountModel {
            id: Uuid::now_v7(),
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
}
