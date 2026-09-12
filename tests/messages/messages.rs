use std::{num::NonZeroU32, str::FromStr};

use kolib::{
    error::{ExportReaderError, MessageError},
    export_reader::{
        account::models::Account,
        datasets::messages::{
            AttachmentSourceKind, get_message_page_by_conversation, get_messages_by_conversation,
        },
        import,
        pagination::{PageRequest, SortOrder},
    },
    types::{Platform, Timestamp},
};
use uuid::Uuid;

use crate::common::{create_account_in_temp_dir, twitter_dm_fixture};

const CONVERSATION_ID: &str = "1234567891234567890-5555555555555555555";

#[tokio::test]
async fn returns_messages_by_conversation() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let messages = get_messages_by_conversation(&archive, &account, CONVERSATION_ID)
        .await
        .expect("getting messages should succeed");

    assert_eq!(messages.len(), 4);
    assert_eq!(
        messages
            .iter()
            .map(|message| message.record_id())
            .collect::<Vec<_>>(),
        [
            "5000000000000000005",
            "6000000000000000006",
            "7000000000000000007",
            "8000000000000000008",
        ]
    );
    assert!(messages.iter().all(|message| {
        Uuid::from_str(message.id()).is_ok() && message.platform() == Platform::Twitter
    }));

    let plain = &messages[0];
    assert_eq!(plain.conversation_id(), CONVERSATION_ID);
    assert_eq!(plain.sender(), "5555555555555555555");
    assert_eq!(plain.recipient(), Some("1234567891234567890"));
    assert_eq!(
        plain.text(),
        Some("This message establishes a second conversation with the sample archive owner.")
    );
    assert_eq!(
        plain.created_at(),
        Some(Timestamp::from_milliseconds(1_788_213_780_005))
    );
    assert!(plain.reactions().is_empty());
    assert!(plain.edit_history().is_empty());
    assert!(plain.attachments().is_empty());

    let single_edit = &messages[1];
    assert_eq!(single_edit.edit_history().len(), 1);
    assert_eq!(
        single_edit.edit_history()[0].text(),
        "This is the only edit-history entry for this message."
    );
    assert_eq!(
        single_edit.edit_history()[0].created_at(),
        Some(Timestamp::from_milliseconds(1_788_213_840_000))
    );

    let multiple_edits = &messages[2];
    assert_eq!(multiple_edits.edit_history().len(), 2);
    assert_eq!(
        multiple_edits.edit_history()[0].text(),
        "This is the first edit-history entry for the multiple-edit message."
    );
    assert_eq!(
        multiple_edits.edit_history()[1].text(),
        "This is the second edit-history entry for the multiple-edit message."
    );

    let everything = &messages[3];
    assert_eq!(everything.reactions().len(), 2);
    assert_eq!(
        everything.reactions()[0].record_id(),
        Some("8000000000000000001")
    );
    assert_eq!(
        everything.reactions()[0].sender(),
        Some("5555555555555555555")
    );
    assert_eq!(everything.reactions()[0].reaction(), "😮");
    assert_eq!(
        everything.reactions()[0].created_at(),
        Some(Timestamp::from_milliseconds(1_788_214_020_001))
    );
    assert_eq!(
        everything.reactions()[1].record_id(),
        Some("8000000000000000002")
    );
    assert_eq!(
        everything.reactions()[1].sender(),
        Some("1234567891234567890")
    );
    assert_eq!(everything.reactions()[1].reaction(), "❤️");

    assert_eq!(everything.edit_history().len(), 2);
    assert_eq!(
        everything.edit_history()[0].text(),
        "This is the first edit-history entry for the message that has everything."
    );
    assert_eq!(
        everything.edit_history()[0].created_at(),
        Some(Timestamp::from_milliseconds(1_788_214_020_000))
    );
    assert_eq!(
        everything.edit_history()[1].text(),
        "This is the second edit-history entry for the message that has everything."
    );
    assert_eq!(
        everything.edit_history()[1].created_at(),
        Some(Timestamp::from_milliseconds(1_788_214_080_000))
    );

    assert_eq!(everything.attachments().len(), 2);
    assert_eq!(
        everything.attachments()[0].source_kind(),
        AttachmentSourceKind::File
    );
    assert_eq!(
        everything.attachments()[0].source(),
        "8000000000000000008-everything-test-video.mp4"
    );
    let expected_file_path = archive
        .folder()
        .join("accounts")
        .join(account.id().to_string())
        .join("twitter-direct-messages")
        .join("media")
        .join("8000000000000000008-everything-test-video.mp4");
    assert_eq!(
        everything.attachments()[0].full_path(&archive, &account),
        Some(expected_file_path)
    );
    assert_eq!(everything.attachments()[0].created_at(), None);
    assert_eq!(
        everything.attachments()[1].source_kind(),
        AttachmentSourceKind::Url
    );
    assert_eq!(
        everything.attachments()[1].source(),
        "https://youtu.be/dQw4w9WgXcQ"
    );
    assert_eq!(
        everything.attachments()[1].full_path(&archive, &account),
        None
    );
}

#[tokio::test]
async fn paginates_messages_from_oldest_to_newest() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let page_size = NonZeroU32::new(3).unwrap();

    let first_page = get_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        PageRequest::new(0, page_size),
    )
    .await
    .expect("getting the first message page should succeed");

    assert_eq!(first_page.page_index(), 0);
    assert_eq!(first_page.page_size(), 3);
    assert_eq!(first_page.total_items(), 4);
    assert_eq!(first_page.total_pages(), 2);
    assert!(first_page.has_next_page());
    assert_eq!(
        first_page
            .items()
            .iter()
            .map(|message| message.record_id())
            .collect::<Vec<_>>(),
        [
            "5000000000000000005",
            "6000000000000000006",
            "7000000000000000007",
        ]
    );

    let second_page = get_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        PageRequest::new(1, page_size),
    )
    .await
    .expect("getting the second message page should succeed");

    assert_eq!(second_page.page_index(), 1);
    assert!(!second_page.has_next_page());
    assert_eq!(
        second_page
            .items()
            .iter()
            .map(|message| message.record_id())
            .collect::<Vec<_>>(),
        ["8000000000000000008"]
    );
}

#[tokio::test]
async fn paginates_messages_from_newest_to_oldest() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let page_size = NonZeroU32::new(3).unwrap();

    let first_page = get_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        PageRequest::new(0, page_size).with_order(SortOrder::NewestFirst),
    )
    .await
    .expect("getting the first newest-first message page should succeed");

    assert_eq!(
        first_page
            .items()
            .iter()
            .map(|message| message.record_id())
            .collect::<Vec<_>>(),
        [
            "8000000000000000008",
            "7000000000000000007",
            "6000000000000000006",
        ]
    );

    let second_page = get_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        PageRequest::new(1, page_size).with_order(SortOrder::NewestFirst),
    )
    .await
    .expect("getting the second newest-first message page should succeed");

    assert_eq!(
        second_page
            .items()
            .iter()
            .map(|message| message.record_id())
            .collect::<Vec<_>>(),
        ["5000000000000000005"]
    );
}

#[tokio::test]
async fn returns_error_for_invalid_paginated_conversation() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let result = get_message_page_by_conversation(
        &archive,
        &account,
        "invalid-conversation",
        PageRequest::new(0, NonZeroU32::new(3).unwrap()),
    )
    .await;
    let expected_account_id = account.id().to_string();

    assert!(
        matches!(
            &result,
            Err(ExportReaderError::Message(
                MessageError::ConversationNotFound {
                    account_id,
                    conversation_id,
                }
            )) if account_id == &expected_account_id
                && conversation_id == "invalid-conversation"
        ),
        "unexpected result: {result:?}"
    );
}

#[tokio::test]
async fn returns_error_for_invalid_conversation() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let result = get_messages_by_conversation(&archive, &account, "invalid-conversation").await;
    let expected_account_id = account.id().to_string();

    assert!(
        matches!(
            &result,
            Err(ExportReaderError::Message(
                MessageError::ConversationNotFound {
                    account_id,
                    conversation_id,
                }
            )) if account_id == &expected_account_id
                && conversation_id == "invalid-conversation"
        ),
        "unexpected result: {result:?}"
    );
}

#[tokio::test]
async fn returns_error_for_conversation_belonging_to_another_account() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let other_account = Account::create(&archive, "other", Platform::Twitter)
        .await
        .expect("creating another Twitter account should succeed");

    let result = get_messages_by_conversation(&archive, &other_account, CONVERSATION_ID).await;
    let expected_account_id = other_account.id().to_string();

    assert!(
        matches!(
            &result,
            Err(ExportReaderError::Message(
                MessageError::ConversationNotFound {
                    account_id,
                    conversation_id,
                }
            )) if account_id == &expected_account_id
                && conversation_id == CONVERSATION_ID
        ),
        "unexpected result: {result:?}"
    );
}
