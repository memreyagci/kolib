use crate::{
    archive::model::Archive, error::ExportReaderError, export_reader::account::models::Account,
};

#[derive(Debug, Clone)]
pub struct ConversationInfo {
    id: String,
    latest_message_text: String,
    latest_message_at: String,
    message_count: i64,
}

impl ConversationInfo {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn latest_message_text(&self) -> &str {
        &self.latest_message_text
    }

    pub fn latest_message_at(&self) -> &str {
        &self.latest_message_at
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

    let conversations = sqlx::query_as!(
        ConversationInfo,
        r#"
      WITH ranked_messages AS (
          SELECT
              conversation_id,
              message_create_id,
              text,
              created_at,
              COUNT(*) OVER (
                  PARTITION BY conversation_id
              ) AS message_count,
              ROW_NUMBER() OVER (
                  PARTITION BY conversation_id
                  ORDER BY created_at DESC, message_create_id DESC
              ) AS message_rank
          FROM twitter_direct_messages
          WHERE account_id = ?
      )
      SELECT
          conversation_id AS "id!",
          text AS "latest_message_text!",
          created_at AS "latest_message_at!",
          message_count AS "message_count!: i64"
      FROM ranked_messages
      WHERE message_rank = 1
      ORDER BY created_at DESC, conversation_id
      "#,
        account_id
    )
    .fetch_all(archive.pool())
    .await?;

    Ok(conversations)
}
