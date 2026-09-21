use crate::{
    archive::model::Archive,
    error::{ExportReaderError, MessageError},
    export_reader::account::models::Account,
};

const CONVERSATION_NOT_FOUND_DATABASE_ERROR: &str =
    "message conversation name must refer to an existing conversation in the account";

pub async fn set_message_conversation_name(
    archive: &Archive,
    account: &Account,
    conversation_id: &str,
    display_name: &str,
) -> Result<(), ExportReaderError> {
    if display_name.trim().is_empty() {
        return Err(MessageError::InvalidConversationDisplayName.into());
    }

    let account_id = account.id().to_string();
    let result = sqlx::query!(
        r#"
        INSERT INTO message_conversation_names (account_id, conversation_id, display_name)
        VALUES (?, ?, ?)
        ON CONFLICT (account_id, conversation_id)
        DO UPDATE SET display_name = excluded.display_name
        "#,
        account_id,
        conversation_id,
        display_name,
    )
    .execute(archive.pool())
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(error)
            if error.as_database_error().is_some_and(|database_error| {
                database_error.message() == CONVERSATION_NOT_FOUND_DATABASE_ERROR
            }) =>
        {
            Err(MessageError::ConversationNotFound {
                account_id,
                conversation_id: conversation_id.to_owned(),
            }
            .into())
        }
        Err(error) => Err(error.into()),
    }
}

pub async fn clear_message_conversation_name(
    archive: &Archive,
    account: &Account,
    conversation_id: &str,
) -> Result<(), ExportReaderError> {
    let account_id = account.id().to_string();

    sqlx::query!(
        r#"
        DELETE FROM message_conversation_names
        WHERE account_id = ?
          AND conversation_id = ?
        "#,
        account_id,
        conversation_id,
    )
    .execute(archive.pool())
    .await?;

    Ok(())
}
