use kolib::{
    export_reader::{
        account::models::Account,
        platforms::twitter::direct_messages::{get_conversations_by_account, import},
    },
    types::Platform,
};

use crate::common::{create_account_in_temp_dir, twitter_dm_fixture};

#[tokio::test]
async fn returns_conversations_by_account() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let conversations = get_conversations_by_account(&archive, &account)
        .await
        .expect("getting Twitter DM conversations should succeed");

    assert_eq!(conversations.len(), 2);

    let conv_1 = &conversations[0];

    assert_eq!(conv_1.id(), "1234567891234567890-5555555555555555555");
    assert_eq!(conv_1.message_count(), 4);
    assert_eq!(conv_1.latest_message_at(), "2026-08-31T22:06:00.008Z");
    assert_eq!(
        conv_1.latest_message_text(),
        "This message has local and external attachments, multiple reactions including a self-reaction, and multiple edit-history entries. https://t.co/everythingmedia https://t.co/everythingurl"
    );

    let conv_2 = &conversations[1];
    assert_eq!(conv_2.id(), "1234567891234567890-9876543219876543210");
    assert_eq!(conv_2.message_count(), 8);
    assert_eq!(conv_2.latest_message_at(), "2026-08-31T22:01:27.041Z");
    assert_eq!(
        conv_2.latest_message_text(),
        "This message has multiple reactions, including a reaction from its sender."
    );
}

#[tokio::test]
async fn does_not_return_another_accounts_conversations() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;

    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");

    let other_account = Account::create(&archive, "other", Platform::Twitter)
        .await
        .expect("creating another Twitter account should succeed");

    let conversations = get_conversations_by_account(&archive, &other_account)
        .await
        .expect("getting Twitter DM conversations should succeed");

    assert!(conversations.is_empty());
}
