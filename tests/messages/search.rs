use std::num::NonZeroU32;

use kolib::{
    error::{ExportReaderError, MessageError},
    export_reader::{
        datasets::messages::{locate_message, search_message_page_by_conversation},
        import,
        pagination::{PageRequest, SortOrder},
    },
    types::{Platform, Timestamp},
};

use crate::common::{create_account_in_temp_dir, twitter_dm_fixture};

const CONVERSATION_ID: &str = "1234567891234567890-5555555555555555555";

fn first_search_page() -> PageRequest {
    PageRequest::new(0, NonZeroU32::new(20).unwrap())
}

#[tokio::test]
async fn matches_the_start_of_words_without_matching_inner_substrings() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let prefix_page = search_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        "establish",
        first_search_page(),
    )
    .await
    .expect("searching by a word prefix should succeed");
    assert_eq!(prefix_page.items().len(), 1);

    let substring_page = search_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        "tablish",
        first_search_page(),
    )
    .await
    .expect("searching by an inner substring should succeed");
    assert!(substring_page.items().is_empty());

    sqlx::query("VACUUM")
        .execute(archive.pool())
        .await
        .expect("vacuuming the archive should succeed");

    let page_after_vacuum = search_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        "establish",
        first_search_page(),
    )
    .await
    .expect("searching after vacuuming should succeed");
    assert_eq!(page_after_vacuum.items().len(), 1);
    assert_eq!(
        page_after_vacuum.items()[0].id(),
        prefix_page.items()[0].id()
    );
}

#[tokio::test]
async fn paginates_message_search_results() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let page_size = NonZeroU32::new(1).unwrap();
    let first_page = search_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        "MULTIPLE EDIT-HISTORY",
        PageRequest::new(0, page_size),
    )
    .await
    .expect("searching messages should succeed");

    assert_eq!(first_page.total_items(), 2);
    assert_eq!(first_page.total_pages(), 2);
    assert_eq!(first_page.items().len(), 1);
    assert!(first_page.has_next_page());
    assert_eq!(first_page.items()[0].conversation_id(), CONVERSATION_ID);
    assert!(
        first_page.items()[0]
            .text()
            .is_some_and(|text| text.contains("multiple"))
    );
    assert_eq!(first_page.items()[0].sender(), "5555555555555555555");
    assert_eq!(
        first_page.items()[0].created_at(),
        Some(Timestamp::from_milliseconds(1_788_213_900_007))
    );

    let second_page = search_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        "MULTIPLE EDIT-HISTORY",
        PageRequest::new(1, page_size),
    )
    .await
    .expect("searching the second result page should succeed");
    assert_eq!(second_page.items().len(), 1);
    assert!(!second_page.has_next_page());

    let newest_first = search_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        "MULTIPLE EDIT-HISTORY",
        PageRequest::new(0, page_size).with_order(SortOrder::NewestFirst),
    )
    .await
    .expect("searching newest-first should succeed");
    assert_eq!(newest_first.items()[0].id(), second_page.items()[0].id());
}

#[tokio::test]
async fn returns_no_hits_for_text_in_another_conversation() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let page = search_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        "plain message without reactions",
        first_search_page(),
    )
    .await
    .expect("searching messages should succeed");

    assert!(page.items().is_empty());
    assert_eq!(page.total_items(), 0);
}

#[tokio::test]
async fn locates_message_for_current_pagination_order() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let hit = search_message_page_by_conversation(
        &archive,
        &account,
        CONVERSATION_ID,
        "establishes a second conversation",
        first_search_page(),
    )
    .await
    .expect("searching messages should succeed")
    .into_items()
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
