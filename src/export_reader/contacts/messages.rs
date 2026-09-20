use std::str::FromStr;

use crate::{
    archive::model::Archive,
    error::ContactError,
    export_reader::{
        account::models::Account,
        contacts::models::{Contact, ContactId},
    },
};

const PARTICIPANT_NOT_SENDER_DATABASE_ERROR: &str =
    "message contact assignment participant must be an existing sender in the conversation";

impl Contact {
    pub async fn assign_message_participant(
        &self,
        archive: &Archive,
        account: &Account,
        conversation_id: &str,
        participant_key: &str,
    ) -> Result<(), ContactError> {
        let account_id = account.id().to_string();
        let contact_id = self.id().to_string();
        let result = sqlx::query!(
            r#"
            INSERT INTO message_contact_assignments (
              account_id,
              conversation_id,
              participant_key,
              contact_id
            )
            VALUES (?, ?, ?, ?)
            ON CONFLICT (account_id, conversation_id, participant_key)
            DO UPDATE SET contact_id = excluded.contact_id
            "#,
            account_id,
            conversation_id,
            participant_key,
            contact_id,
        )
        .execute(archive.pool())
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(error)
                if error.as_database_error().is_some_and(|database_error| {
                    database_error.message() == PARTICIPANT_NOT_SENDER_DATABASE_ERROR
                }) =>
            {
                Err(ContactError::MessageParticipantNotFound {
                    account_id,
                    conversation_id: conversation_id.to_owned(),
                    participant_key: participant_key.to_owned(),
                })
            }
            Err(error) => Err(error.into()),
        }
    }

    pub async fn unassign_message_participant(
        &self,
        archive: &Archive,
        account: &Account,
        conversation_id: &str,
        participant_key: &str,
    ) -> Result<(), ContactError> {
        let account_id = account.id().to_string();
        let contact_id = self.id().to_string();

        sqlx::query!(
            r#"
            DELETE FROM message_contact_assignments
            WHERE account_id = ?
              AND conversation_id = ?
              AND participant_key = ?
              AND contact_id = ?
            "#,
            account_id,
            conversation_id,
            participant_key,
            contact_id,
        )
        .execute(archive.pool())
        .await?;

        Ok(())
    }
}

pub async fn get_message_participant_contact(
    archive: &Archive,
    account: &Account,
    conversation_id: &str,
    participant_key: &str,
) -> Result<Option<Contact>, ContactError> {
    let account_id = account.id().to_string();
    let row = sqlx::query!(
        r#"
        SELECT contact.id, contact.name, contact.is_me
        FROM message_contact_assignments AS assignment
        JOIN contacts AS contact
          ON contact.id = assignment.contact_id
        WHERE assignment.account_id = ?
          AND assignment.conversation_id = ?
          AND assignment.participant_key = ?
        "#,
        account_id,
        conversation_id,
        participant_key,
    )
    .fetch_optional(archive.pool())
    .await?;

    row.map(|row| {
        Ok(Contact::new(
            ContactId::from_str(&row.id)?,
            row.name,
            row.is_me != 0,
        ))
    })
    .transpose()
}
