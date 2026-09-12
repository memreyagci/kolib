use sqlx::SqliteConnection;

use crate::{error::ExportReaderError, export_reader::datasets::messages::rows::MessageImportRows};

pub(crate) async fn insert(
    connection: &mut SqliteConnection,
    rows: &MessageImportRows,
) -> Result<(), ExportReaderError> {
    for message in &rows.messages {
        sqlx::query(
            r#"
            INSERT INTO messages (
                id,
                account_id,
                platform,
                conversation_id,
                record_id,
                sender,
                recipient,
                text,
                created_at_ms
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&message.id)
        .bind(&message.account_id)
        .bind(message.platform.as_ref())
        .bind(&message.conversation_id)
        .bind(&message.record_id)
        .bind(&message.sender)
        .bind(&message.recipient)
        .bind(&message.text)
        .bind(message.created_at_ms)
        .execute(&mut *connection)
        .await?;
    }

    for reaction in &rows.reactions {
        sqlx::query(
            r#"
            INSERT INTO message_reactions (
                main_id,
                ordinal,
                record_id,
                sender,
                reaction,
                created_at_ms
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&reaction.main_id)
        .bind(reaction.ordinal)
        .bind(&reaction.record_id)
        .bind(&reaction.sender)
        .bind(&reaction.reaction)
        .bind(reaction.created_at_ms)
        .execute(&mut *connection)
        .await?;
    }

    for edit in &rows.edits {
        sqlx::query(
            r#"
            INSERT INTO message_edits (main_id, ordinal, text, created_at_ms)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&edit.main_id)
        .bind(edit.ordinal)
        .bind(&edit.text)
        .bind(edit.created_at_ms)
        .execute(&mut *connection)
        .await?;
    }

    for attachment in &rows.file_attachments {
        sqlx::query(
            r#"
            INSERT INTO message_file_attachments (
                main_id,
                ordinal,
                filename,
                created_at_ms
            )
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&attachment.main_id)
        .bind(attachment.ordinal)
        .bind(&attachment.filename)
        .bind(attachment.created_at_ms)
        .execute(&mut *connection)
        .await?;
    }

    for attachment in &rows.link_attachments {
        sqlx::query(
            r#"
            INSERT INTO message_link_attachments (
                main_id,
                ordinal,
                url,
                created_at_ms
            )
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&attachment.main_id)
        .bind(attachment.ordinal)
        .bind(&attachment.url)
        .bind(attachment.created_at_ms)
        .execute(&mut *connection)
        .await?;
    }

    Ok(())
}
