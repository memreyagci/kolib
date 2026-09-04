use kolib::{
    error::ExportReaderError, export_reader::platforms::twitter::direct_messages::import,
    types::Platform,
};

use crate::common::{create_account_in_temp_dir, twitter_dm_fixture};

#[tokio::test]
async fn imports_comprehensive_export() {
    // TODO: Also verify that:
    // - One dataset was added.
    // - Its type is "direct-messages.js".
    // - Existing media fixtures were copied, and intentionally missing one was not.
    // - Getter results, when implemented, returns the expected counts.
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");
}

#[tokio::test]
async fn imports_empty_export() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("empty"))
        .await
        .expect("empty Twitter DM import should succeed");
}

#[tokio::test]
async fn rejects_invalid_json() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    let result = import(&archive, &account, twitter_dm_fixture("invalid_json")).await;

    assert!(
        matches!(result, Err(ExportReaderError::SerdeError(_))),
        "unexpected result: {result:?}"
    );
}

#[tokio::test]
async fn rejects_missing_required_fields() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    let result = import(
        &archive,
        &account,
        twitter_dm_fixture("missing_required_fields"),
    )
    .await;

    assert!(
        matches!(result, Err(ExportReaderError::SerdeError(_))),
        "unexpected result: {result:?}"
    );
}

#[tokio::test]
async fn accepts_missing_optional_fields() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(
        &archive,
        &account,
        twitter_dm_fixture("missing_optional_fields"),
    )
    .await
    .expect("missing_optional_fields Twitter DM import should succeed");
}

#[tokio::test]
async fn rolls_back_import_with_duplicate_message_ids() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    let result = import(&archive, &account, twitter_dm_fixture("duplicate_ids")).await;

    assert!(
        matches!(result, Err(ExportReaderError::SqlxError(_))),
        "unexpected result: {result:?}"
    );

    let datasets = account
        .get_datasets(&archive)
        .await
        .expect("failed to retrieve datasets");

    assert!(
        datasets.is_empty(),
        "failed import unexpectedly left {} dataset(s) behind",
        datasets.len()
    );
}
