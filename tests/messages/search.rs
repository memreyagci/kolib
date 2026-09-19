use std::num::NonZeroU32;

use kolib::{
    error::{ExportReaderError, MessageError},
    export_reader::{
        datasets::messages::{locate_message, search_messages_by_conversation},
        import,
        pagination::SortOrder,
    },
    types::{Platform, Timestamp},
};

use crate::common::{create_account_in_temp_dir, twitter_dm_fixture};

const CONVERSATION_ID: &str = "1234567891234567890-5555555555555555555";

#[tokio::test]
async fn matches_the_start_of_words_without_matching_inner_substrings() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let prefix_hits =
        search_messages_by_conversation(&archive, &account, CONVERSATION_ID, "establish")
            .await
            .expect("searching by a word prefix should succeed");
    assert_eq!(prefix_hits.len(), 1);

    let substring_hits =
        search_messages_by_conversation(&archive, &account, CONVERSATION_ID, "tablish")
            .await
            .expect("searching by an inner substring should succeed");
    assert!(substring_hits.is_empty());

    sqlx::query("VACUUM")
        .execute(archive.pool())
        .await
        .expect("vacuuming the archive should succeed");

    let hits_after_vacuum =
        search_messages_by_conversation(&archive, &account, CONVERSATION_ID, "establish")
            .await
            .expect("searching after vacuuming should succeed");
    assert_eq!(hits_after_vacuum.len(), 1);
    assert_eq!(hits_after_vacuum[0].id(), prefix_hits[0].id());
}

#[tokio::test]
async fn searches_message_text_within_conversation() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let hits = search_messages_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        "MULTIPLE EDIT-HISTORY",
    )
    .await
    .expect("searching messages should succeed");

    assert_eq!(hits.len(), 2);
    assert!(hits.iter().all(|hit| {
        hit.conversation_id() == CONVERSATION_ID
            && hit.text().is_some_and(|text| text.contains("multiple"))
    }));
    assert_eq!(hits[0].sender(), "5555555555555555555");
    assert_eq!(
        hits[0].created_at(),
        Some(Timestamp::from_milliseconds(1_788_213_900_007))
    );
}

#[tokio::test]
async fn returns_no_hits_for_text_in_another_conversation() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let hits = search_messages_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        "plain message without reactions",
    )
    .await
    .expect("searching messages should succeed");

    assert!(hits.is_empty());
}

#[tokio::test]
async fn locates_message_for_current_pagination_order() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let hit = search_messages_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        "establishes a second conversation",
    )
    .await
    .expect("searching messages should succeed")
    .into_iter()
    .next()
    .expect("the searched message should exist");
    let page_size = NonZeroU32::new(3).unwrap();

    let oldest_first = locate_message(
        &archive,
        &account,
        hit.id(),
        page_size,
        SortOrder::OldestFirst,
    )
    .await
    .expect("locating the message oldest-first should succeed");
    assert_eq!(oldest_first.conversation_id(), CONVERSATION_ID);
    assert_eq!(oldest_first.page_index(), 0);

    let newest_first = locate_message(
        &archive,
        &account,
        hit.id(),
        page_size,
        SortOrder::NewestFirst,
    )
    .await
    .expect("locating the message newest-first should succeed");
    assert_eq!(newest_first.conversation_id(), CONVERSATION_ID);
    assert_eq!(newest_first.page_index(), 1);
}

#[tokio::test]
async fn returns_error_when_located_message_does_not_exist() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    let message_id = "missing-message";

    let result = locate_message(
        &archive,
        &account,
        message_id,
        NonZeroU32::new(20).unwrap(),
        SortOrder::OldestFirst,
    )
    .await;
    let expected_account_id = account.id().to_string();

    assert!(
        matches!(
            &result,
            Err(ExportReaderError::Message(MessageError::NotFound {
                account_id,
                message_id: actual_message_id,
            })) if account_id == &expected_account_id && actual_message_id == message_id
        ),
        "unexpected result: {result:?}"
    );
}
