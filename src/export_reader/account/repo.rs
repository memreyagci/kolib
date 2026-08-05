use std::{fs, path::Path, str::FromStr};

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    archive::model::Archive, error::AccountError, export_reader::account::models::Account,
    types::Platform,
};

pub async fn create(
    archive: &Archive,
    name: &str,
    platform: Platform,
) -> Result<Account, AccountError> {
    let account = Account::new(Uuid::now_v7(), name.to_string(), platform);

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

/// Takes the Account model (consumes it, thus users won't end up continuing to have the
/// pre-rename instance) and returns the renamed one.
pub async fn rename(
    pool: &SqlitePool,
    account: Account,
    new_name: &str,
) -> Result<Account, AccountError> {
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

pub async fn delete(pool: &SqlitePool, account: Account) -> Result<(), AccountError> {
    // TODO: make sure doing so also deletes all related fields from account_datasets and platform
    // file-related tables
    let _ = sqlx::query!(
        "DELETE FROM accounts WHERE id = ?",
        account.id().to_string()
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Returns an Account instance by its id, which is the unique identifier.
pub async fn get_by_id(pool: &SqlitePool, id: Uuid) -> Result<Account, AccountError> {
    // TODO: Make errors more verbose, e.g "account with ID: {id} doesn't exist"
    let account = sqlx::query!(
        "SELECT id, name, platform FROM accounts WHERE id = ?;",
        id.to_string()
    )
    .fetch_one(pool)
    .await?;

    Ok(Account::new(
        Uuid::from_str(&account.id)?,
        account.name,
        Platform::from_str(&account.platform)?,
    ))
}

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Account>, AccountError> {
    // TODO: If not exists, it will panic. Handle it accordingly.
    let rows = sqlx::query!("SELECT * FROM accounts;")
        .fetch_all(pool)
        .await?;

    let mut accounts: Vec<Account> = Vec::new();

    for row in rows {
        accounts.push(Account::new(
            Uuid::from_str(&row.id)?,
            row.name,
            Platform::from_str(&row.platform)?,
        ));
    }

    Ok(accounts)
}
