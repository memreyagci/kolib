use crate::{
    archive::model::Archive, error::ExportReaderError, export_reader::account::models::Account,
};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ConversationInfo {
    id: String,
    latest_message_text: Option<String>,
    latest_message_at_ms: Option<i64>,
    message_count: i64,
}

impl ConversationInfo {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn latest_message_text(&self) -> Option<&str> {
        self.latest_message_text.as_deref()
    }

    pub fn latest_message_at_ms(&self) -> Option<i64> {
        self.latest_message_at_ms
    }

    pub fn message_count(&self) -> i64 {
        self.message_count
    }
}

pub async fn get_conversations_by_account(
    archive: &Archive,
    account: &Account,
) -> Result<Vec<ConversationInfo>, ExportReaderError> {
    let account_id = account.id().to_string();

    let conversations = sqlx::query_as::<_, ConversationInfo>(
        r#"
        WITH ranked_messages AS (
          SELECT
            conversation_id,
            record_id,
            text,
            created_at_ms,
            COUNT(*) OVER (
              PARTITION BY conversation_id
            ) AS message_count,
            ROW_NUMBER() OVER (
              PARTITION BY conversation_id
              ORDER BY created_at_ms IS NULL, created_at_ms DESC, record_id DESC
            ) AS message_rank
          FROM messages
          WHERE account_id = ?
        )
        SELECT
          conversation_id AS id,
          text AS latest_message_text,
          created_at_ms AS latest_message_at_ms,
          message_count
        FROM ranked_messages
        WHERE message_rank = 1
        ORDER BY
          latest_message_at_ms IS NULL,
          latest_message_at_ms DESC,
          conversation_id
        "#,
    )
    .bind(account_id)
    .fetch_all(archive.pool())
    .await?;

    Ok(conversations)
}
