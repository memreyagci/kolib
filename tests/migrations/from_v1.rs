use std::{num::NonZeroU32, str::FromStr};

use kolib::{
    archive::model::Archive,
    export_reader::{
        account::models::{Account, AccountId},
        datasets::{
            DatasetType,
            messages::{
                AttachmentSourceKind, get_conversations_by_account, get_messages_by_conversation,
                search_message_page_by_conversation,
            },
        },
        pagination::PageRequest,
    },
    types::Platform,
};

use crate::utils::{copy_fixture_to_temp, message_row_counts, migration_versions};

const ACCOUNT_ID: &str = "01a072e0-a4d6-744c-a632-39857ae991ff";
const COMPREHENSIVE_MESSAGE_CONVERSATION_ID: &str = "1234567891234567890-5555555555555555555";
const FIXTURE_RELATIVE_PATH: &str = "archives/v1";

#[tokio::test]
async fn migrates_v1_archive_to_latest() {
    let (_guard, archive_path) = copy_fixture_to_temp(FIXTURE_RELATIVE_PATH);
    let archive = Archive::open(&archive_path)
        .await
        .expect("migrating the copied v1 archive should succeed");

    assert_eq!(migration_versions(&archive).await, [1, 2]);

    let (messages, reactions, edits, attachments) = message_row_counts(&archive).await;
    assert_eq!(messages, 12);
    assert_eq!(reactions, 5);
    assert_eq!(edits, 5);
    assert_eq!(attachments, 7);

    let deprecated_migration_table_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '__drizzle_migrations'",
    )
    .fetch_one(archive.pool())
    .await
    .expect("checking for the deprecated migration table should succeed");
    assert_eq!(deprecated_migration_table_count, 0);

    let (created_at_ms, storage_type) = sqlx::query_as::<_, (i64, String)>(
        r#"
        SELECT created_at_ms, typeof(created_at_ms)
        FROM messages
        WHERE record_id = '1000000000000000001'
        "#,
    )
    .fetch_one(archive.pool())
    .await
    .expect("reading a migrated message timestamp should succeed");
    assert_eq!(created_at_ms, 1_788_213_531_197);
    assert_eq!(storage_type, "integer");

    let reactions = sqlx::query_as::<_, (String, String, i64)>(
        r#"
        SELECT record_id, reaction, created_at_ms
        FROM message_reactions
        ORDER BY record_id
        "#,
    )
    .fetch_all(archive.pool())
    .await
    .expect("reading migrated reactions should succeed");
    assert_eq!(
        reactions,
        [
            (
                "3000000000000000001".to_owned(),
                "👍".to_owned(),
                1_788_213_660_001,
            ),
            (
                "4000000000000000001".to_owned(),
                "😂".to_owned(),
                1_788_213_720_001,
            ),
            (
                "4000000000000000002".to_owned(),
                "❤️".to_owned(),
                1_788_213_721_002,
            ),
            (
                "8000000000000000001".to_owned(),
                "surprised".to_owned(),
                1_788_214_020_001,
            ),
            (
                "8000000000000000002".to_owned(),
                "❤️".to_owned(),
                1_788_214_021_002,
            ),
        ]
    );

    let edits = sqlx::query_as::<_, (String, i64, String, i64)>(
        r#"
        SELECT
            message.record_id,
            edit.ordinal,
            edit.text,
            edit.created_at_ms
        FROM message_edits AS edit
        JOIN messages AS message ON message.id = edit.main_id
        ORDER BY message.record_id, edit.ordinal
        "#,
    )
    .fetch_all(archive.pool())
    .await
    .expect("reading migrated edit history should succeed");
    assert_eq!(edits.len(), 5);
    assert_eq!(
        edits.first(),
        Some(&(
            "6000000000000000006".to_owned(),
            0,
            "This is the only edit-history entry for this message.".to_owned(),
            1_788_213_810_000,
        ))
    );
    assert_eq!(edits.last().map(|edit| edit.3), Some(1_788_213_930_000));

    let attachments = sqlx::query_as::<_, (String, i64, String, String)>(
        r#"
        SELECT
            message.record_id,
            attachment.ordinal,
            attachment.kind,
            attachment.source
        FROM (
            SELECT main_id, ordinal, 'file' AS kind, filename AS source
            FROM message_file_attachments
            UNION ALL
            SELECT main_id, ordinal, 'url' AS kind, url AS source
            FROM message_link_attachments
        ) AS attachment
        JOIN messages AS message ON message.id = attachment.main_id
        ORDER BY message.record_id, attachment.ordinal
        "#,
    )
    .fetch_all(archive.pool())
    .await
    .expect("reading migrated attachments should succeed");
    assert_eq!(attachments.len(), 7);
    assert!(attachments.iter().any(|attachment| {
        attachment.0 == "8000000000000000008"
            && attachment.1 == 0
            && attachment.2 == "file"
            && attachment.3 == "8000000000000000008-everything-test-video.mp4"
    }));
    assert!(attachments.iter().any(|attachment| {
        attachment.0 == "8000000000000000008"
            && attachment.1 == 1
            && attachment.2 == "url"
            && attachment.3 == "https://youtu.be/dQw4w9WgXcQ"
    }));

    let foreign_key_violations = sqlx::query("PRAGMA foreign_key_check")
        .fetch_all(archive.pool())
        .await
        .expect("checking migrated foreign keys should succeed");
    assert!(foreign_key_violations.is_empty());

    let account_id = AccountId::from_str(ACCOUNT_ID).expect("fixture account ID should be valid");
    let account = Account::get_by_id(archive.pool(), &account_id)
        .await
        .expect("the migrated account should remain readable");
    assert_eq!(account.name(), "my_old_account");
    assert_eq!(account.platform(), &Platform::Twitter);

    let datasets = account
        .get_datasets(&archive)
        .await
        .expect("the migrated account's datasets should remain readable");
    assert_eq!(datasets.len(), 1);
    assert_eq!(datasets[0].account_id(), account.id());
    assert_eq!(datasets[0].dataset_type(), &DatasetType::Messages);

    let migrated_dataset_path = archive
        .folder()
        .join("accounts")
        .join(account.id().to_string())
        .join("twitter-direct-messages");
    assert!(
        migrated_dataset_path
            .join("raw")
            .join("direct-messages.js")
            .is_file()
    );

    let migrated_media_path = migrated_dataset_path.join("media");
    for filename in [
        "1111111111111111111-abc12D--efghIJklmN3-OPRsTU4vy5ZZ6_aBcDeFHIj7klMNo-.mp4",
        "8000000000000000008-everything-test-video.mp4",
        "9000000000000000009-ImageToken.jpg",
        "9100000000000000010-VideoToken.mp4",
    ] {
        assert!(
            migrated_media_path.join(filename).is_file(),
            "expected media file `{filename}` to survive migration"
        );
    }

    assert!(
        !migrated_media_path
            .join("9200000000000000011-IntentionallyMissingMedia.mp4")
            .exists(),
        "intentionally missing media file was unexpectedly created during migration"
    );

    let conversations = get_conversations_by_account(&archive, &account)
        .await
        .expect("migrated conversations should remain readable");
    assert_eq!(conversations.len(), 2);
    assert_eq!(
        conversations
            .iter()
            .map(|conversation| conversation.message_count())
            .sum::<i64>(),
        12
    );

    let messages =
        get_messages_by_conversation(&archive, &account, COMPREHENSIVE_MESSAGE_CONVERSATION_ID)
            .await
            .expect("migrated messages should remain readable");
    let comprehensive_message = messages
        .iter()
        .find(|message| message.record_id() == "8000000000000000008")
        .expect("the fixture's comprehensive message should survive migration");
    assert_eq!(comprehensive_message.reactions().len(), 2);
    assert_eq!(comprehensive_message.edit_history().len(), 2);
    assert_eq!(comprehensive_message.attachments().len(), 2);
    assert_eq!(
        comprehensive_message.attachments()[0].source_kind(),
        AttachmentSourceKind::File
    );
    assert_eq!(
        comprehensive_message.attachments()[1].source_kind(),
        AttachmentSourceKind::Url
    );

    let search_page = search_message_page_by_conversation(
        &archive,
        &account,
        COMPREHENSIVE_MESSAGE_CONVERSATION_ID,
        "establish",
        PageRequest::new(0, NonZeroU32::new(20).unwrap()),
    )
    .await
    .expect("searching migrated messages should succeed");
    assert_eq!(search_page.total_items(), 1);
}

#[tokio::test]
async fn reopening_migrated_v1_archive_does_not_reapply_migration() {
    let (_guard, archive_path) = copy_fixture_to_temp(FIXTURE_RELATIVE_PATH);
    let archive = Archive::open(&archive_path)
        .await
        .expect("migrating the copied v1 archive should succeed");

    archive.close().await;

    let reopened = Archive::open(&archive_path)
        .await
        .expect("reopening the migrated archive should succeed");
    assert_eq!(migration_versions(&reopened).await, [1, 2]);

    let (messages, reactions, edits, attachments) = message_row_counts(&reopened).await;
    assert_eq!(messages, 12);
    assert_eq!(reactions, 5);
    assert_eq!(edits, 5);
    assert_eq!(attachments, 7);
}
