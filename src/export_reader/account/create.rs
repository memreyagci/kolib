use std::fs;

use crate::{
    archive::model::Archive,
    error::AccountError,
    export_reader::account::models::{Account, AccountId},
    types::Platform,
};

impl Account {
    pub async fn create(
        archive: &Archive,
        name: &str,
        platform: Platform,
    ) -> Result<Self, AccountError> {
        Self::validate_name(name)?;

        let account = Self::new(AccountId::new(), name.to_string(), platform);

        let mut tx = archive.pool().begin().await?;

        let _ = sqlx::query!(
            "INSERT INTO accounts (id, name, platform) VALUES (?, ?, ?)",
            account.id().to_string(),
            account.name(),
            account.platform().to_string()
        )
        .execute(&mut *tx)
        .await?;

        fs::create_dir_all(
            archive
                .folder()
                .join("accounts")
                .join(account.id().to_string()),
        )?;
        tx.commit().await?;

        Ok(account)
    }
}
