use std::fs;

use kolib::{
    error::ExportReaderError, export_reader::platforms::twitter::direct_messages::import,
    types::Platform,
};

use crate::common::{create_account_in_temp_dir, twitter_dm_fixture};

#[tokio::test]
async fn imports_comprehensive_export() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    let fixture = twitter_dm_fixture("comprehensive");

    import(&archive, &account, &fixture)
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let datasets = account
        .get_datasets(&archive)
        .await
        .expect("getting the imported account's datasets should succeed");
    assert_eq!(datasets.len(), 1);
    assert_eq!(datasets[0].account_id(), account.id());
    assert_eq!(datasets[0].dataset_type(), "direct-messages.js");

    let imported_dataset_path = archive
        .folder()
        .join("accounts")
        .join(account.id().to_string())
        .join("twitter-direct-messages");
    let imported_raw_file = imported_dataset_path.join("raw").join("direct-messages.js");

    assert_eq!(
        fs::read(imported_raw_file).expect("reading the copied raw export should succeed"),
        fs::read(fixture).expect("reading the source export fixture should succeed")
    );

    let imported_media_path = imported_dataset_path.join("media");
    for filename in [
        "1111111111111111111-abc12D--efghIJklmN3-OPRsTU4vy5ZZ6_aBcDeFHIj7klMNo-.mp4",
        "8000000000000000008-everything-test-video.mp4",
        "9000000000000000009-ImageToken.jpg",
        "9100000000000000010-VideoToken.mp4",
    ] {
        assert!(
            imported_media_path.join(filename).is_file(),
            "expected media file `{filename}` to be copied"
        );
    }

    assert!(
        !imported_media_path
            .join("9200000000000000011-IntentionallyMissingMedia.mp4")
            .exists(),
        "intentionally missing media file was unexpectedly created"
    );
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
        matches!(result, Err(ExportReaderError::Serde(_))),
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
        matches!(result, Err(ExportReaderError::Serde(_))),
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
        matches!(result, Err(ExportReaderError::Sqlx(_))),
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
