use std::path::PathBuf;

use sqlx::{SqlitePool, sqlite::SqliteRow};
use uuid::Uuid;

use crate::{error::AccountError, export_reader::account::models::AccountModel, types::Platform};

#[derive(Debug)]
pub struct AccountRepository {
    pool: SqlitePool,
    archive_folder: PathBuf,
}

impl AccountRepository {
    pub(crate) fn new(pool: SqlitePool, archive_folder: PathBuf) -> Self {
        Self {
            pool,
            archive_folder,
        }
    }

    pub async fn create(
        &self,
        name: &str,
        platform: Platform,
    ) -> Result<AccountModel, AccountError> {
        let account = AccountModel::new(Uuid::now_v7(), name.to_string(), platform);

        let _ = sqlx::query::<_>("INSERT INTO accounts (id, name, platform) VALUES (?, ?, ?)")
            .bind(account.id())
            .bind(account.name())
            .bind(account.platform().to_string())
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(account)
    }

    /// Takes the Account model (consumes it, thus users won't end up continuing to have the
    /// pre-rename instance) and returns the renamed one.
    pub async fn rename(
        &self,
        account: AccountModel,
        new_name: &str,
    ) -> Result<AccountModel, AccountError> {
        // TODO: Make errors more verbose, e.g "account with ID: {id} doesn't exist"
        let _ = sqlx::query::<_>("UPDATE accounts SET name = ? WHERE id = ?")
            .bind(new_name)
            .bind(account.id().to_string())
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(AccountModel::new(
            account.id(),
            new_name.to_owned(),
            account.platform().to_owned(),
        ))
    }

    pub async fn delete(&self, account: AccountModel) -> Result<(), AccountError> {
        // TODO: Make errors more verbose, e.g "account with ID: {id} doesn't exist"
        // TODO: make sure doing so also deletes all related fields from account_datasets and platform
        // file-related tables
        let _ = sqlx::query::<_>("DELETE FROM accounts WHERE id = ?")
            .bind(account.id().to_string())
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }

    /// Returns an Account instance by its id, which is the unique identifier.
    pub async fn get_by_id(&self, id: Uuid) -> SqliteRow {
        // TODO: Make errors more verbose, e.g "account with ID: {id} doesn't exist"
        let account = sqlx::query::<_>("SELECT id, name, platform FROM accounts WHERE id = ?;")
            .bind(id.to_string())
            .fetch_one(&self.pool)
            .await
            .unwrap();

        account
    }

    pub async fn get_all(&self) -> Vec<SqliteRow> {
        // TODO: If not exists, it will panic. Handle it accordingly.
        let rows = sqlx::query::<_>("SELECT * FROM accounts;")
            .fetch_all(&self.pool)
            .await
            .unwrap();

        rows
    }
}
