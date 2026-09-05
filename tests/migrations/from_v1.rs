use std::str::FromStr;

use kolib::{
    archive::model::Archive,
    export_reader::{
        account::models::{Account, AccountId},
        platforms::twitter::direct_messages::{
            AttachmentSourceKind, get_conversations_by_account, get_messages_by_conversation,
        },
    },
    types::Platform,
};

use crate::utils::{copy_fixture_to_temp, migration_versions, twitter_dm_row_counts};

const ACCOUNT_ID: &str = "01a071da-f2bb-74ee-9734-8b795190756b";
const COMPREHENSIVE_MESSAGE_CONVERSATION_ID: &str = "1234567891234567890-5555555555555555555";
const FIXTURE_RELATIVE_PATH: &str = "archives/v1";

#[tokio::test]
async fn migrates_v1_archive_to_latest() {
    let (_guard, archive_path) = copy_fixture_to_temp(FIXTURE_RELATIVE_PATH);
    let archive = Archive::open(&archive_path)
        .await
        .expect("migrating the copied v1 archive should succeed");

    assert_eq!(migration_versions(&archive).await, [1, 2]);

    let (messages, reactions, edits, attachments) = twitter_dm_row_counts(&archive).await;
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

    let (created_at, storage_type) = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT created_at, typeof(created_at)
        FROM twitter_direct_messages
        WHERE message_create_id = '1000000000000000001'
        "#,
    )
    .fetch_one(archive.pool())
    .await
    .expect("reading a migrated message timestamp should succeed");
    assert_eq!(created_at, "2026-08-31T21:58:51.197Z");
    assert_eq!(storage_type, "text");

    let reactions = sqlx::query_as::<_, (String, String, String)>(
        r#"
        SELECT event_id, reaction_key, created_at
        FROM twitter_dm_reactions
        ORDER BY event_id
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
                "agree".to_owned(),
                "2026-08-31T22:01:00.001Z".to_owned(),
            ),
            (
                "4000000000000000001".to_owned(),
                "funny".to_owned(),
                "2026-08-31T22:02:00.001Z".to_owned(),
            ),
            (
                "4000000000000000002".to_owned(),
                "like".to_owned(),
                "2026-08-31T22:02:01.002Z".to_owned(),
            ),
            (
                "8000000000000000001".to_owned(),
                "surprised".to_owned(),
                "2026-08-31T22:07:00.001Z".to_owned(),
            ),
            (
                "8000000000000000002".to_owned(),
                "like".to_owned(),
                "2026-08-31T22:07:01.002Z".to_owned(),
            ),
        ]
    );

    let edits = sqlx::query_as::<_, (String, i64, String, String)>(
        r#"
        SELECT
            message.message_create_id,
            edit.ordinal,
            edit.edited_text,
            edit.created_at_sec
        FROM twitter_dm_edit_history AS edit
        JOIN twitter_direct_messages AS message ON message.id = edit.main_id
        ORDER BY message.message_create_id, edit.ordinal
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
            "1788213840".to_owned(),
        ))
    );
    assert_eq!(edits.last().map(|edit| edit.3.as_str()), Some("1788214080"));

    let attachments = sqlx::query_as::<_, (String, i64, String, String)>(
        r#"
        SELECT
            message.message_create_id,
            attachment.ordinal,
            attachment.source_kind,
            attachment.source
        FROM twitter_dm_attachments AS attachment
        JOIN twitter_direct_messages AS message ON message.id = attachment.main_id
        ORDER BY message.message_create_id, attachment.ordinal
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
    assert_eq!(datasets[0].account_id(), &account_id);
    assert_eq!(datasets[0].dataset_type(), "direct-messages.js");

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
        .find(|message| message.id() == "8000000000000000008")
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

    let (messages, reactions, edits, attachments) = twitter_dm_row_counts(&reopened).await;
    assert_eq!(messages, 12);
    assert_eq!(reactions, 5);
    assert_eq!(edits, 5);
    assert_eq!(attachments, 7);
}
