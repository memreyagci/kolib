use kolib::{
    archive::model::Archive,
    error::ContactError,
    export_reader::{
        contacts::{get_message_participant_contact, models::Contact},
        import,
    },
    types::Platform,
};
use uuid::Uuid;

use crate::common::{create_account_in_temp_dir, create_archive_in_temp_dir, twitter_dm_fixture};

mod common;

const CONVERSATION_ID: &str = "1234567891234567890-9876543219876543210";
const PARTICIPANT_KEY: &str = "9876543219876543210";

#[tokio::test]
async fn creates_and_reuses_builtin_me_contact() {
    let (_guard, archive_path, archive) = create_archive_in_temp_dir().await;
    let me = Contact::get_me(&archive)
        .await
        .expect("getting the built-in Me contact should succeed");

    assert_eq!(me.name(), "Me");
    assert!(me.is_me());
    assert_eq!(
        Uuid::parse_str(&me.id().to_string())
            .expect("the built-in contact ID should be a UUID")
            .get_version_num(),
        7
    );

    assert_eq!(
        Contact::get_all(&archive)
            .await
            .expect("getting contacts should succeed")
            .len(),
        1
    );
}

#[tokio::test]
async fn creates_renames_and_deletes_contact() {
    let (_guard, _, archive) = create_archive_in_temp_dir().await;
    let mut contact = Contact::create(&archive, "Jack")
        .await
        .expect("creating a contact should succeed");
    let contact_id = contact.id().clone();

    contact
        .rename(&archive, "Jackson")
        .await
        .expect("renaming a contact should succeed");
    assert_eq!(contact.name(), "Jackson");

    let fetched = Contact::get_by_id(&archive, &contact_id)
        .await
        .expect("getting a contact by ID should succeed");
    assert_eq!(fetched.name(), "Jackson");
    assert!(!fetched.is_me());

    contact
        .delete(&archive)
        .await
        .expect("deleting a contact should succeed");
    let result = Contact::get_by_id(&archive, &contact_id).await;
    let expected_id = contact_id.to_string();
    assert!(
        matches!(
            &result,
            Err(ContactError::NotFound { contact_id }) if contact_id == &expected_id
        ),
        "unexpected result: {result:?}"
    );
}

#[tokio::test]
async fn rejects_invalid_contact_names() {
    let (_guard, _, archive) = create_archive_in_temp_dir().await;

    for invalid_name in ["", " ", "\t\n"] {
        let result = Contact::create(&archive, invalid_name).await;
        assert!(
            matches!(&result, Err(ContactError::InvalidName)),
            "unexpected result: {result:?}"
        );
    }
}

#[tokio::test]
async fn prevents_renaming_or_deleting_me() {
    let (_guard, _, archive) = create_archive_in_temp_dir().await;
    let mut me = Contact::get_me(&archive)
        .await
        .expect("getting the built-in Me contact should succeed");
    let me_id = me.id().clone();

    let rename_result = me.rename(&archive, "Owner").await;
    assert!(
        matches!(rename_result, Err(ContactError::CannotRenameMe)),
        "unexpected result: {rename_result:?}"
    );

    let delete_result = me.delete(&archive).await;
    assert!(
        matches!(delete_result, Err(ContactError::CannotDeleteMe)),
        "unexpected result: {delete_result:?}"
    );

    let me_id_string = me_id.to_string();
    let database_rename_result = sqlx::query("UPDATE contacts SET name = 'Owner' WHERE id = ?")
        .bind(&me_id_string)
        .execute(archive.pool())
        .await;
    assert_eq!(
        database_rename_result
            .expect_err("the database should reject renaming Me")
            .as_database_error()
            .expect("renaming Me should return a database error")
            .message(),
        "the built-in Me contact cannot be modified"
    );

    let database_delete_result = sqlx::query("DELETE FROM contacts WHERE id = ?")
        .bind(&me_id_string)
        .execute(archive.pool())
        .await;
    assert_eq!(
        database_delete_result
            .expect_err("the database should reject deleting Me")
            .as_database_error()
            .expect("deleting Me should return a database error")
            .message(),
        "the built-in Me contact cannot be deleted"
    );

    let fetched = Contact::get_by_id(&archive, &me_id)
        .await
        .expect("Me should remain after rejected modifications");
    assert_eq!(fetched.name(), "Me");
}

#[tokio::test]
async fn assigns_reassigns_and_unassigns_message_participant() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");
    let first = Contact::create(&archive, "First")
        .await
        .expect("creating the first contact should succeed");
    let second = Contact::create(&archive, "Second")
        .await
        .expect("creating the second contact should succeed");

    first
        .assign_message_participant(&archive, &account, CONVERSATION_ID, PARTICIPANT_KEY)
        .await
        .expect("assigning a message participant should succeed");
    let assigned =
        get_message_participant_contact(&archive, &account, CONVERSATION_ID, PARTICIPANT_KEY)
            .await
            .expect("getting a participant's contact should succeed")
            .expect("the participant should be assigned");
    assert_eq!(assigned.id(), first.id());

    second
        .assign_message_participant(&archive, &account, CONVERSATION_ID, PARTICIPANT_KEY)
        .await
        .expect("reassigning a message participant should succeed");

    first
        .unassign_message_participant(&archive, &account, CONVERSATION_ID, PARTICIPANT_KEY)
        .await
        .expect("unassigning through the old contact should be harmless");
    let reassigned =
        get_message_participant_contact(&archive, &account, CONVERSATION_ID, PARTICIPANT_KEY)
            .await
            .expect("getting the reassigned participant's contact should succeed")
            .expect("the participant should remain assigned");
    assert_eq!(reassigned.id(), second.id());

    second
        .unassign_message_participant(&archive, &account, CONVERSATION_ID, PARTICIPANT_KEY)
        .await
        .expect("unassigning a message participant should succeed");
    assert!(
        get_message_participant_contact(&archive, &account, CONVERSATION_ID, PARTICIPANT_KEY,)
            .await
            .expect("getting an unassigned participant's contact should succeed")
            .is_none()
    );
}

#[tokio::test]
async fn rejects_message_identity_that_is_not_a_sender() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");
    let contact = Contact::create(&archive, "Contact")
        .await
        .expect("creating a contact should succeed");

    let result = contact
        .assign_message_participant(&archive, &account, CONVERSATION_ID, "not-a-sender")
        .await;
    let expected_account_id = account.id().to_string();

    assert!(
        matches!(
            &result,
            Err(ContactError::MessageParticipantNotFound {
                account_id,
                conversation_id,
                participant_key,
            }) if account_id == &expected_account_id
                && conversation_id == CONVERSATION_ID
                && participant_key == "not-a-sender"
        ),
        "unexpected result: {result:?}"
    );
}

#[tokio::test]
async fn deleting_contact_removes_its_message_assignments() {
    let (_guard, _, archive, account) = create_account_in_temp_dir(Platform::Twitter).await;
    import(&archive, &account, twitter_dm_fixture("comprehensive"))
        .await
        .expect("comprehensive Twitter DM import should succeed");
    let contact = Contact::create(&archive, "Contact")
        .await
        .expect("creating a contact should succeed");

    contact
        .assign_message_participant(&archive, &account, CONVERSATION_ID, PARTICIPANT_KEY)
        .await
        .expect("assigning a message participant should succeed");
    contact
        .delete(&archive)
        .await
        .expect("deleting the contact should succeed");

    assert!(
        get_message_participant_contact(&archive, &account, CONVERSATION_ID, PARTICIPANT_KEY,)
            .await
            .expect("getting the deleted contact's participant should succeed")
            .is_none()
    );
}
