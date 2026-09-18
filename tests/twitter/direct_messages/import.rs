use std::fs;

use kolib::{
    error::ExportReaderError,
    export_reader::{datasets::DatasetType, import},
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
    assert_eq!(datasets[0].dataset_type(), &DatasetType::Messages);

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
async fn rejects_import_when_dataset_already_exists() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    let fixture = twitter_dm_fixture("comprehensive");

    import(&archive, &account, &fixture)
        .await
        .expect("initial Twitter DM import should succeed");

    let result = import(&archive, &account, &fixture).await;
    let expected_account_id = account.id().to_string();

    assert!(
        matches!(
            &result,
            Err(ExportReaderError::DatasetAlreadyExists {
                account_id,
                dataset_type,
            }) if account_id == &expected_account_id && dataset_type == "messages"
        ),
        "unexpected result: {result:?}"
    );

    let message_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM messages")
        .fetch_one(archive.pool())
        .await
        .expect("counting messages after the rejected import should succeed");
    assert_eq!(message_count, 12);
}

#[tokio::test]
async fn accepts_empty_export_without_creating_dataset_or_directory() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("empty"))
        .await
        .expect("empty Twitter DM import should succeed");

    let datasets = account
        .get_datasets(&archive)
        .await
        .expect("getting datasets after an empty import should succeed");
    assert!(datasets.is_empty()); // No dataset should be created for an empty export file.
    assert!(
        !archive
            .folder()
            .join("accounts")
            .join(account.id().to_string())
            .join("twitter-direct-messages")
            .exists(),
        "empty import unexpectedly created a dataset directory"
    );
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
async fn rejects_unexpected_filename() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    let unexpected_file = archive.folder().join("messages.js");

    fs::copy(twitter_dm_fixture("comprehensive"), &unexpected_file)
        .expect("copying the fixture with an unexpected filename should succeed");

    let result = import(&archive, &account, unexpected_file).await;

    assert!(
        matches!(
            &result,
            Err(ExportReaderError::UnexpectedFilename { expected, actual })
                if expected == "direct-messages.js" && actual == "messages.js"
        ),
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

    let (messages, reactions, edits, attachments) = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        r#"
            SELECT
                (SELECT COUNT(*) FROM messages),
                (SELECT COUNT(*) FROM message_reactions),
                (SELECT COUNT(*) FROM message_edits),
                (
                    (SELECT COUNT(*) FROM message_file_attachments)
                    + (SELECT COUNT(*) FROM message_link_attachments)
                )
            "#,
    )
    .fetch_one(archive.pool())
    .await
    .expect("counting rows after the failed import should succeed");

    assert_eq!(messages, 0);
    assert_eq!(reactions, 0);
    assert_eq!(edits, 0);
    assert_eq!(attachments, 0);

    assert!(
        !archive
            .folder()
            .join("accounts")
            .join(account.id().to_string())
            .join("twitter-direct-messages")
            .exists(),
        "failed import unexpectedly left its dataset directory behind"
    );
}
