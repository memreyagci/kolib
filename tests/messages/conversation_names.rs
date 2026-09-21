use kolib::{
    error::{ExportReaderError, MessageError},
    export_reader::{
        account::models::Account,
        datasets::messages::{
            clear_message_conversation_name, get_conversations_by_account,
            set_message_conversation_name,
        },
        import,
    },
    types::Platform,
};

use crate::common::{create_account_in_temp_dir, twitter_dm_fixture};

const CONVERSATION_ID: &str = "1234567891234567890-9876543219876543210";

#[tokio::test]
async fn sets_replaces_and_clears_conversation_name() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    for name in ["Friends", "Old friends"] {
        set_message_conversation_name(&archive, &account, CONVERSATION_ID, name)
            .await
            .expect("setting a conversation name should succeed");

        let conversations = get_conversations_by_account(&archive, &account)
            .await
            .expect("getting conversations should succeed");
        assert_eq!(
            conversations
                .iter()
                .find(|conversation| conversation.id() == CONVERSATION_ID)
                .expect("the named conversation should be returned")
                .display_name(),
            Some(name)
        );
        assert!(
            conversations
                .iter()
                .filter(|conversation| conversation.id() != CONVERSATION_ID)
                .all(|conversation| conversation.display_name().is_none())
        );
    }

    clear_message_conversation_name(&archive, &account, CONVERSATION_ID)
        .await
        .expect("clearing a conversation name should succeed");
    let conversations = get_conversations_by_account(&archive, &account)
        .await
        .expect("getting conversations after clearing the name should succeed");
    assert!(
        conversations
            .iter()
            .all(|conversation| conversation.display_name().is_none())
    );
}

#[tokio::test]
async fn rejects_invalid_conversation_names_and_unknown_conversations() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    for invalid_name in ["", " ", "\t\n"] {
        let result =
            set_message_conversation_name(&archive, &account, CONVERSATION_ID, invalid_name).await;
        assert!(
            matches!(
                result,
                Err(ExportReaderError::Message(
                    MessageError::InvalidConversationDisplayName
                ))
            ),
            "unexpected result: {result:?}"
        );
    }

    let result = set_message_conversation_name(&archive, &account, "missing", "Unknown").await;
    let expected_account_id = account.id().to_string();
    assert!(
        matches!(
            &result,
            Err(ExportReaderError::Message(MessageError::ConversationNotFound {
                account_id,
                conversation_id,
            })) if account_id == &expected_account_id && conversation_id == "missing"
        ),
        "unexpected result: {result:?}"
    );
}

#[tokio::test]
async fn conversation_names_are_scoped_to_accounts() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("first account's Twitter DM import should succeed");
    let other_account = Account::create(&archive, "other", Platform::Twitter)
        .await
        .expect("creating another account should succeed");
    import(
        &archive,
        &other_account,
        twitter_dm_fixture("comprehensive"),
    )
    .await
    .expect("other account's Twitter DM import should succeed");

    set_message_conversation_name(&archive, &account, CONVERSATION_ID, "Friends")
        .await
        .expect("naming the first account's conversation should succeed");

    let other_conversations = get_conversations_by_account(&archive, &other_account)
        .await
        .expect("getting the other account's conversations should succeed");
    assert!(
        other_conversations
            .iter()
            .all(|conversation| conversation.display_name().is_none())
    );
}
