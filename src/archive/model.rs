use std::path::PathBuf;

use sqlx::SqlitePool;

use crate::export_reader::{account::models::Account, datasets::DatasetType};

#[derive(Debug)]
pub struct Archive {
    pool: SqlitePool,
    folder: PathBuf,
}

impl Archive {
    pub fn new(pool: SqlitePool, folder: PathBuf) -> Self {
        Self { pool, folder }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn folder(&self) -> &std::path::Path {
        &self.folder
    }

    pub(crate) fn account_directory(&self, account: &Account) -> PathBuf {
        self.folder()
            .join("accounts")
            .join(account.id().to_string())
    }

    pub(crate) fn dataset_directory(
        &self,
        account: &Account,
        dataset_type: DatasetType,
    ) -> PathBuf {
        let directory_name = match dataset_type {
            DatasetType::Messages => "twitter-direct-messages",
        };

        self.account_directory(account).join(directory_name)
    }

    pub async fn close(self) {
        self.pool.close().await;
    }
}
