use std::fmt::Debug;

use kolib::{
    error::AccountError,
    export_reader::account::models::{Account, AccountId},
    types::Platform,
};

use crate::common::{create_account_in_temp_dir, create_archive_in_temp_dir};

mod common;

fn assert_account_not_found<T: Debug>(
    result: Result<T, AccountError>,
    expected_account_id: &AccountId,
) {
    let expected_account_id = expected_account_id.to_string();

    assert!(
        matches!(
            &result,
            Err(AccountError::NotFound { account_id })
                if account_id == &expected_account_id
        ),
        "unexpected result: {result:?}"
    );
}

#[tokio::test]
async fn creates_account() {
    let (_guard, _, archive) = create_archive_in_temp_dir().await;

    let account = Account::create(&archive, "test", Platform::Twitter)
        .await
        .expect("creating an account should succeed");

    assert_eq!(account.name(), "test");
    assert_eq!(account.platform(), &Platform::Twitter);
    assert!(
        archive
            .folder()
            .join("accounts")
            .join(account.id().to_string())
            .is_dir()
    );
}

#[tokio::test]
async fn rejects_invalid_names_when_creating_account() {
    let (_guard, _, archive) = create_archive_in_temp_dir().await;

    for invalid_name in ["", " ", "\t\n"] {
        let result = Account::create(&archive, invalid_name, Platform::Twitter).await;
        assert!(
            matches!(&result, Err(AccountError::InvalidName)),
            "unexpected result: {result:?}"
        );
    }

    let accounts = Account::get_all(archive.pool())
        .await
        .expect("getting accounts after rejected creation attempts should succeed");
    assert!(accounts.is_empty());
}

#[tokio::test]
async fn gets_account_by_id() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    let fetched = Account::get_by_id(archive.pool(), account.id())
        .await
        .expect("getting an account by ID should succeed");

    assert_eq!(fetched.id(), account.id());
    assert_eq!(fetched.name(), account.name());
    assert_eq!(fetched.platform(), account.platform());
}

#[tokio::test]
async fn renames_account() {
    let (_guard, _, archive, mut account) = create_account_in_temp_dir(Platform::Twitter).await;
    let account_id = account.id().clone();

    account
        .rename(archive.pool(), "renamed")
        .await
        .expect("renaming an account should succeed");

    assert_eq!(account.id(), &account_id);
    assert_eq!(account.name(), "renamed");

    let fetched = Account::get_by_id(archive.pool(), &account_id)
        .await
        .expect("getting the renamed account should succeed");

    assert_eq!(fetched.name(), "renamed");
}

#[tokio::test]
async fn rejects_invalid_names_when_renaming_account() {
    let (_guard, _, archive, mut account) = create_account_in_temp_dir(Platform::Twitter).await;
    let account_id = account.id().clone();

    for invalid_name in ["", " ", "\t\n"] {
        let result = account.rename(archive.pool(), invalid_name).await;

        assert!(
            matches!(&result, Err(AccountError::InvalidName)),
            "unexpected result: {result:?}"
        );
        assert_eq!(account.name(), "test");
    }

    let fetched = Account::get_by_id(archive.pool(), &account_id)
        .await
        .expect("getting the account after rejected rename attempts should succeed");
    assert_eq!(fetched.name(), "test");
}

#[tokio::test]
async fn deletes_account() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    let account_id = account.id().clone();

    account
        .delete(&archive)
        .await
        .expect("deleting an account should succeed");

    let result = Account::get_by_id(archive.pool(), &account_id).await;
    assert_account_not_found(result, &account_id);
}

#[tokio::test]
async fn returns_not_found_when_renaming_deleted_account() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    let account_id = account.id().clone();
    let mut stale_account = Account::get_by_id(archive.pool(), &account_id)
        .await
        .expect("getting a second account instance should succeed");

    account
        .delete(&archive)
        .await
        .expect("deleting an account should succeed");

    let result = stale_account.rename(archive.pool(), "renamed").await;

    assert_account_not_found(result, &account_id);
    assert_eq!(stale_account.name(), "test");
}

#[tokio::test]
async fn returns_not_found_when_deleting_deleted_account() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    let account_id = account.id().clone();
    let stale_account = Account::get_by_id(archive.pool(), &account_id)
        .await
        .expect("getting a second account instance should succeed");

    account
        .delete(&archive)
        .await
        .expect("deleting an account should succeed");

    let result = stale_account.delete(&archive).await;
    assert_account_not_found(result, &account_id);
}
