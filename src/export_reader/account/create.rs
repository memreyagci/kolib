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

#[cfg(test)]
mod tests {
    use sqlx::Error::RowNotFound;

    use crate::{
        error::AccountError, export_reader::account::models::Account,
        test_helpers::init_archive_in_temp_dir, types::Platform,
    };

    #[tokio::test]
    async fn account_crud_succeeds() {
        let (_guard, _, archive) = init_archive_in_temp_dir().await;
        let acc_result = Account::create(&archive, "test", Platform::Twitter).await;

        assert!(acc_result.is_ok());

        let acc = acc_result.unwrap();
        let acc_id = acc.id().clone();

        assert!(Account::get_by_id(archive.pool(), acc.id()).await.is_ok());
        assert!(acc.delete(&archive).await.is_ok());

        // After deletion, we should not be able to fetch the deleted Account by id.
        assert!(matches!(
            Account::get_by_id(archive.pool(), &acc_id).await,
            Err(AccountError::Sqlx(RowNotFound))
        ));
    }
}
