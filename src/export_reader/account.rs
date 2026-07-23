//! An account in kolib is where data from an account of a platform is be stored in.
//!
//! For instance, a user can create an account with the name @my_old_acc for Twitter, in that account,
//! only pre-defined export/takeout files knowing to be coming from Twitter will be accepted.
//! When user requests to see the data from @my_old_acc, they will see direct messages, tweets,
//! following/followers list of that account.
//!
//! An account is a requirement when an export/takeout file from a platform is to be imported.

use std::str::FromStr;

use sqlx::Row;
use uuid::Uuid;

use crate::{error::AccountError, types::Platform};

/// An account consist of three parts: id, name, and paltform. They are mandatory fields in its relative table in
/// the database, but not here necessarily, since the implementation has functions that does not require all of
/// them to run it, such as for getting an account by id, or getting all accounts.
#[derive(Debug)]
pub struct Account {
    id: Uuid,
    name: Option<String>,
    platform: Option<Platform>,
}

// TODO: Decide what functions that insert/update/delete db fields should return. Would an OK(()) response
// be sufficient, or is there another convention?
impl Account {
    /// To create a new account and save it to a database, name and platform must be set. uuid is
    /// automatically generated.
    ///
    /// ```
    /// use kolib::export_reader::account::Account;
    /// use kolib::types::Platform;
    /// let account = Account::new().name("@my_old_acc".to_string()).platform(Platform::Twitter);
    /// ```
    pub fn new() -> Self {
        Account {
            id: Uuid::now_v7(),
            name: None,
            platform: None,
        }
    }

    // Setters
    pub fn set_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    pub fn set_platform(mut self, platform: Platform) -> Self {
        self.platform = Some(platform);
        self
    }

    // Getters
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn name(&self) -> Result<&String, AccountError> {
        if let Some(n) = &self.name {
            Ok(n)
        } else {
            Err(AccountError::AccountNameNull)
        }
    }
    pub fn platform(&self) -> Result<&Platform, String> {
        if let Some(p) = &self.platform {
            Ok(p)
        } else {
            Err("Platform is not set".to_string())
        }
    }

    /// To save an account to the database, all you need to do is to run this function on an instance and
    /// provide an sqlx connection.
    pub async fn save_to_db(&self, conn: &sqlx::SqlitePool) -> Result<Self, String> {
        let row =
            sqlx::query::<_>("INSERT INTO accounts (id, name, platform) VALUES (?, ?, ?) RETURNING id, name, platform;")
                .bind(self.id.to_string())
                .bind(&self.name)
                .bind(self.platform.as_ref().map(|p| p.as_ref()))
                .fetch_one(conn)
                .await
                .unwrap();

        Ok(Account {
            id: Uuid::parse_str(row.get("id")).unwrap(),
            name: Some(row.get("name")),
            platform: Some(Platform::from_str(row.get("platform")).unwrap()),
        })
    }

    pub async fn rename(id: Uuid, new_name: &str, conn: &sqlx::SqlitePool) {
        // TODO: If not exists, it will panic. Handle it accordingly.
        let result = sqlx::query::<_>("UPDATE accounts SET name = ? WHERE id = ?")
            .bind(new_name)
            .bind(id.to_string())
            .fetch_one(conn)
            .await
            .unwrap();
    }

    pub async fn delete(id: Uuid, conn: &sqlx::SqlitePool) {
        // TODO: If not exists, it will panic. Handle it accordingly.
        let result = sqlx::query::<_>("DELETE FROM accounts WHERE id = ?")
            .bind(id.to_string())
            .fetch_one(conn)
            .await
            .unwrap();

        // TODO: make sure doing so also deletes all related fields from account_datasets and platform
        // file-related tables
    }

    /// Returns an Account instance by its id, which is the unique identifier.
    pub async fn get_by_id(id: Uuid, conn: &sqlx::SqlitePool) -> Self {
        // TODO: If not exists, it will panic. Handle it accordingly.
        let account = sqlx::query::<_>("SELECT id, name, platform FROM accounts WHERE id = ?;")
            .bind(id.to_string())
            .fetch_one(conn)
            .await
            .unwrap();

        let result = Account {
            id: Uuid::parse_str(account.get("id")).unwrap(),
            name: Some(account.get("name")),
            platform: Some(Platform::from_str(account.get("platform")).unwrap()),
        };

        result
    }

    pub async fn get_all(conn: &sqlx::SqlitePool) -> Vec<Self> {
        // TODO: If not exists, it will panic. Handle it accordingly.
        let rows = sqlx::query::<_>("SELECT * FROM accounts;")
            .fetch_all(conn)
            .await
            .unwrap();

        let mut accounts: Vec<Self> = Vec::new();

        for row in rows {
            accounts.push(Account {
                id: Uuid::parse_str(row.get("id")).unwrap(),
                name: Some(row.get("name")),
                platform: Some(Platform::from_str(row.get("platform")).unwrap()),
            });
        }

        accounts
    }
}
