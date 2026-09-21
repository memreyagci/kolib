use crate::{
    archive::model::Archive, error::ExportReaderError, export_reader::account::models::Account,
    types::Timestamp,
};

#[derive(Debug, Clone)]
pub struct ConversationInfo {
    id: String,
    display_name: Option<String>,
    latest_message_text: Option<String>,
    latest_message_at: Option<Timestamp>,
    message_count: i64,
}

impl ConversationInfo {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn latest_message_text(&self) -> Option<&str> {
        self.latest_message_text.as_deref()
    }

    pub fn latest_message_at(&self) -> Option<Timestamp> {
        self.latest_message_at
    }

    pub fn message_count(&self) -> i64 {
        self.message_count
    }
}

#[derive(sqlx::FromRow)]
struct ConversationQueryRow {
    id: String,
    display_name: Option<String>,
    latest_message_text: Option<String>,
    latest_message_at_ms: Option<i64>,
    message_count: i64,
}

pub async fn get_conversations_by_account(
    archive: &Archive,
    account: &Account,
) -> Result<Vec<ConversationInfo>, ExportReaderError> {
    let account_id = account.id().to_string();

    let rows = sqlx::query_as::<_, ConversationQueryRow>(
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
              ORDER BY
                created_at_ms IS NULL,
                created_at_ms DESC,
                record_id DESC,
                id DESC
            ) AS message_rank
          FROM messages
          WHERE account_id = ?
        )
        SELECT
          ranked.conversation_id AS id,
          conversation_name.display_name,
          ranked.text AS latest_message_text,
          ranked.created_at_ms AS latest_message_at_ms,
          ranked.message_count
        FROM ranked_messages AS ranked
        LEFT JOIN message_conversation_names AS conversation_name
          ON conversation_name.account_id = ?
         AND conversation_name.conversation_id = ranked.conversation_id
        WHERE ranked.message_rank = 1
        ORDER BY
          latest_message_at_ms IS NULL,
          latest_message_at_ms DESC,
          ranked.conversation_id
        "#,
    )
    .bind(&account_id)
    .bind(&account_id)
    .fetch_all(archive.pool())
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ConversationInfo {
            id: row.id,
            display_name: row.display_name,
            latest_message_text: row.latest_message_text,
            latest_message_at: row.latest_message_at_ms.map(Timestamp::from),
            message_count: row.message_count,
        })
        .collect())
}
