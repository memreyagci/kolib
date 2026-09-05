use kolib::{
    error::{ExportReaderError, TwitterError},
    export_reader::{
        account::models::Account,
        platforms::twitter::direct_messages::{
            AttachmentSourceKind, get_messages_by_conversation, import,
        },
    },
    types::Platform,
};

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
        .expect("getting Twitter DM messages should succeed");

    assert_eq!(messages.len(), 4);
    assert_eq!(
        messages
            .iter()
            .map(|message| message.id())
            .collect::<Vec<_>>(),
        [
            "5000000000000000005",
            "6000000000000000006",
            "7000000000000000007",
            "8000000000000000008",
        ]
    );

    let plain = &messages[0];
    assert_eq!(plain.conversation_id(), CONVERSATION_ID);
    assert_eq!(plain.sender_id(), "5555555555555555555");
    assert_eq!(plain.recipient_id(), "1234567891234567890");
    assert_eq!(
        plain.text(),
        "This message establishes a second conversation with the sample archive owner."
    );
    assert_eq!(plain.created_at(), "2026-08-31T22:03:00.005Z");
    assert!(plain.reactions().is_empty());
    assert!(plain.edit_history().is_empty());
    assert!(plain.attachments().is_empty());

    let single_edit = &messages[1];
    assert_eq!(single_edit.edit_history().len(), 1);
    assert_eq!(
        single_edit.edit_history()[0].edited_text(),
        "This is the only edit-history entry for this message."
    );
    assert_eq!(single_edit.edit_history()[0].created_at_sec(), "1788213840");

    let multiple_edits = &messages[2];
    assert_eq!(multiple_edits.edit_history().len(), 2);
    assert_eq!(
        multiple_edits.edit_history()[0].edited_text(),
        "This is the first edit-history entry for the multiple-edit message."
    );
    assert_eq!(
        multiple_edits.edit_history()[1].edited_text(),
        "This is the second edit-history entry for the multiple-edit message."
    );

    let everything = &messages[3];
    assert_eq!(everything.reactions().len(), 2);
    assert_eq!(everything.reactions()[0].event_id(), "8000000000000000001");
    assert_eq!(everything.reactions()[0].sender_id(), "5555555555555555555");
    assert_eq!(everything.reactions()[0].reaction_key(), "😮");
    assert_eq!(
        everything.reactions()[0].created_at(),
        "2026-08-31T22:07:00.001Z"
    );
    assert_eq!(everything.reactions()[1].event_id(), "8000000000000000002");
    assert_eq!(everything.reactions()[1].sender_id(), "1234567891234567890");
    assert_eq!(everything.reactions()[1].reaction_key(), "❤️");

    assert_eq!(everything.edit_history().len(), 2);
    assert_eq!(
        everything.edit_history()[0].edited_text(),
        "This is the first edit-history entry for the message that has everything."
    );
    assert_eq!(everything.edit_history()[0].created_at_sec(), "1788214020");
    assert_eq!(
        everything.edit_history()[1].edited_text(),
        "This is the second edit-history entry for the message that has everything."
    );
    assert_eq!(everything.edit_history()[1].created_at_sec(), "1788214080");

    assert_eq!(everything.attachments().len(), 2);
    assert_eq!(
        everything.attachments()[0].source_kind(),
        AttachmentSourceKind::File
    );
    assert_eq!(
        everything.attachments()[0].source(),
        "8000000000000000008-everything-test-video.mp4"
    );
    assert_eq!(
        everything.attachments()[1].source_kind(),
        AttachmentSourceKind::Url
    );
    assert_eq!(
        everything.attachments()[1].source(),
        "https://youtu.be/dQw4w9WgXcQ"
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
            Err(ExportReaderError::Twitter(
                TwitterError::ConversationNotFound {
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
            Err(ExportReaderError::Twitter(
                TwitterError::ConversationNotFound {
                    account_id,
                    conversation_id,
                }
            )) if account_id == &expected_account_id
                && conversation_id == CONVERSATION_ID
        ),
        "unexpected result: {result:?}"
    );
}
