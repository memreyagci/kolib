use std::str::FromStr;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    archive::model::Archive,
    error::AccountError,
    export_reader::account::models::{Account, Dataset},
    types::Platform,
};

impl Account {
    /// Takes the Account model (consumes it, thus users won't end up continuing to have the
    /// pre-rename instance) and returns the renamed one.
    pub async fn rename(
        pool: &SqlitePool,
        account: Self,
        new_name: &str,
    ) -> Result<Self, AccountError> {
        Self::validate_name(new_name)?;

        // TODO: Make errors more verbose, e.g "account with ID: {id} doesn't exist"
        let _ = sqlx::query!(
            "UPDATE accounts SET name = ? WHERE id = ?",
            new_name,
            account.id().to_string()
        )
        .execute(pool)
        .await?;

        Ok(Account::new(
            account.id(),
            new_name.to_owned(),
            account.platform().to_owned(),
        ))
    }

    pub async fn delete(self, archive: &Archive) -> Result<(), AccountError> {
        // TODO: make sure doing so also deletes all related fields from account_datasets and platform
        // file-related tables
        let _ = sqlx::query!("DELETE FROM accounts WHERE id = ?", self.id().to_string())
            .execute(archive.pool())
            .await?;

        Ok(())
    }

    /// Returns an Account instance by its id, which is the unique identifier.
    pub async fn get_by_id(pool: &SqlitePool, id: Uuid) -> Result<Self, AccountError> {
        // TODO: Make errors more verbose, e.g "account with ID: {id} doesn't exist"
        let account = sqlx::query!(
            "SELECT id, name, platform FROM accounts WHERE id = ?;",
            id.to_string()
        )
        .fetch_one(pool)
        .await?;

        Ok(Self::new(
            Uuid::from_str(&account.id)?,
            account.name,
            Platform::from_str(&account.platform)?,
        ))
    }

    pub async fn get_datasets(&self, archive: &Archive) -> Result<Vec<Dataset>, AccountError> {
        let rows = sqlx::query!(
            "SELECT account_id, dataset_type FROM account_datasets WHERE account_id = ?;",
            self.id()
        )
        .fetch_all(archive.pool())
        .await?;

        let datasets: Vec<Dataset> = rows
            .into_iter()
            .map(|row| -> Result<Dataset, AccountError> {
                Ok(Dataset::new(
                    Uuid::from_str(&row.account_id)?,
                    row.dataset_type,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(datasets)
    }

    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Self>, AccountError> {
        let rows = sqlx::query!("SELECT id, name, platform FROM accounts;")
            .fetch_all(pool)
            .await?;

        let accounts: Vec<Self> = rows
            .into_iter()
            .map(|row| -> Result<Self, AccountError> {
                Ok(Self::new(
                    Uuid::from_str(&row.id)?,
                    row.name,
                    Platform::from_str(&row.platform)?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(accounts)
    }
}
