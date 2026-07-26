use sqlx::{Row, SqlitePool, sqlite::SqliteRow};
use uuid::Uuid;

use crate::{error::AccountError, export_reader::account::models::AccountModel, types::Platform};

#[derive(Debug)]
pub struct AccountRepository {
    pool: SqlitePool,
}

impl AccountRepository {
    pub async fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, name: &str, platform: Platform) -> Result<Uuid, AccountError> {
        let account = AccountModel::new(name.to_string(), platform);

        let result = sqlx::query::<_>("INSERT INTO accounts (id, name, platform) VALUES (?, ?, ?) RETURNING id, name, platform;")
                .bind(account.id())
                .bind(account.name())
                .bind(account.platform().to_string())
                .fetch_one(&self.pool)
               .await.unwrap();

        Ok(result.get("id"))
    }

    pub async fn rename(&self, id: Uuid, new_name: &str) -> Result<(), AccountError> {
        // TODO: Make errors more verbose, e.g "account with ID: {id} doesn't exist"
        let _ = sqlx::query::<_>("UPDATE accounts SET name = ? WHERE id = ?")
            .bind(new_name)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AccountError> {
        // TODO: Make errors more verbose, e.g "account with ID: {id} doesn't exist"
        // TODO: make sure doing so also deletes all related fields from account_datasets and platform
        // file-related tables
        let _ = sqlx::query::<_>("DELETE FROM accounts WHERE id = ?")
            .bind(id.to_string())
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
