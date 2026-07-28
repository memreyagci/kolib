use std::str::FromStr;

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::{error::AccountError, export_reader::account::models::Account, types::Platform};

pub async fn create(
    pool: &SqlitePool,
    name: &str,
    platform: Platform,
) -> Result<Account, AccountError> {
    let account = Account::new(Uuid::now_v7(), name.to_string(), platform);

    let _ = sqlx::query::<_>("INSERT INTO accounts (id, name, platform) VALUES (?, ?, ?)")
        .bind(account.id())
        .bind(account.name())
        .bind(account.platform().to_string())
        .execute(pool)
        .await
        .unwrap();

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
    let _ = sqlx::query::<_>("UPDATE accounts SET name = ? WHERE id = ?")
        .bind(new_name)
        .bind(account.id().to_string())
        .execute(pool)
        .await
        .unwrap();

    Ok(Account::new(
        account.id(),
        new_name.to_owned(),
        account.platform().to_owned(),
    ))
}

pub async fn delete(pool: &SqlitePool, account: Account) -> Result<(), AccountError> {
    // TODO: make sure doing so also deletes all related fields from account_datasets and platform
    // file-related tables
    let _ = sqlx::query::<_>("DELETE FROM accounts WHERE id = ?")
        .bind(account.id().to_string())
        .execute(pool)
        .await
        .unwrap();

    Ok(())
}

/// Returns an Account instance by its id, which is the unique identifier.
pub async fn get_by_id(pool: &SqlitePool, id: Uuid) -> Result<Account, AccountError> {
    // TODO: Make errors more verbose, e.g "account with ID: {id} doesn't exist"
    let account = sqlx::query::<_>("SELECT id, name, platform FROM accounts WHERE id = ?;")
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .unwrap();

    Ok(Account::new(
        Uuid::from_str(account.try_get("id").unwrap()).unwrap(),
        account.try_get("name").unwrap(),
        Platform::from_str(account.try_get("platform").unwrap()).unwrap(),
    ))
}

pub async fn get_all(pool: &SqlitePool) -> Vec<Account> {
    // TODO: If not exists, it will panic. Handle it accordingly.
    let rows = sqlx::query::<_>("SELECT * FROM accounts;")
        .fetch_all(pool)
        .await
        .unwrap();

    let mut accounts: Vec<Account> = Vec::new();

    for row in rows {
        accounts.push(Account::new(
            Uuid::from_str(row.try_get("id").unwrap()).unwrap(),
            row.try_get("name").unwrap(),
            Platform::from_str(row.try_get("platform").unwrap()).unwrap(),
        ));
    }

    accounts
}
