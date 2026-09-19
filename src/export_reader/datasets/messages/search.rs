use std::num::NonZeroU32;

use crate::{
    archive::model::Archive,
    error::{ExportReaderError, MessageError},
    export_reader::{
        account::models::Account,
        pagination::{Page, PageRequest, SortOrder},
    },
    types::Timestamp,
};

#[derive(Debug, Clone)]
pub struct MessageSearchHit {
    id: String,
    conversation_id: String,
    sender: String,
    text: Option<String>,
    created_at: Option<Timestamp>,
}

impl MessageSearchHit {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn conversation_id(&self) -> &str {
        &self.conversation_id
    }

    pub fn sender(&self) -> &str {
        &self.sender
    }

    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    pub fn created_at(&self) -> Option<Timestamp> {
        self.created_at
    }
}

#[derive(Debug, Clone)]
pub struct MessageLocation {
    conversation_id: String,
    page_index: u32,
}

impl MessageLocation {
    pub fn conversation_id(&self) -> &str {
        &self.conversation_id
    }

    pub fn page_index(&self) -> u32 {
        self.page_index
    }
}

pub async fn search_message_page_by_conversation(
    archive: &Archive,
    account: &Account,
    conversation_id: &str,
    query: &str,
    pagination: PageRequest,
) -> Result<Page<MessageSearchHit>, ExportReaderError> {
    let Some(query) = word_prefix_query(query) else {
        return Ok(Page::new(Vec::new(), pagination, 0));
    };
    let account_id = account.id().to_string();
    let total_items = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "total_items!: i64"
        FROM messages_fts
        JOIN messages AS message
          ON message.id = messages_fts.message_id
        WHERE messages_fts MATCH ?
          AND message.account_id = ?
          AND message.conversation_id = ?
        "#,
        query,
        account_id,
        conversation_id,
    )
    .fetch_one(archive.pool())
    .await? as u64;

    let page_index = u64::from(pagination.page_index());
    let page_size = u64::from(pagination.page_size().get());
    let skipped_items = page_index * page_size;

    let (offset, limit) = match pagination.order() {
        SortOrder::OldestFirst => {
            let start = skipped_items.min(total_items);
            let end = (start + page_size).min(total_items);

            (start, end - start)
        }
        SortOrder::NewestFirst => {
            let end = total_items.saturating_sub(skipped_items);
            let start = end.saturating_sub(page_size);

            (start, end - start)
        }
    };
    let offset = offset as i64;
    let limit = limit as i64;

    let rows = sqlx::query!(
        r#"
        SELECT
          message.id AS "id!",
          message.conversation_id AS "conversation_id!",
          message.sender AS "sender!",
          message.text,
          message.created_at_ms
        FROM messages_fts
        JOIN messages AS message
          ON message.id = messages_fts.message_id
        WHERE messages_fts MATCH ?
          AND message.account_id = ?
          AND message.conversation_id = ?
        ORDER BY
          message.created_at_ms,
          message.record_id
        LIMIT ?
        OFFSET ?
        "#,
        query,
        account_id,
        conversation_id,
        limit,
        offset,
    )
    .fetch_all(archive.pool())
    .await?;

    let mut hits = rows
        .into_iter()
        .map(|row| MessageSearchHit {
            id: row.id,
            conversation_id: row.conversation_id,
            sender: row.sender,
            text: row.text,
            created_at: row.created_at_ms.map(Timestamp::from),
        })
        .collect::<Vec<_>>();

    if pagination.order() == SortOrder::NewestFirst {
        hits.reverse();
    }

    Ok(Page::new(hits, pagination, total_items))
}

fn word_prefix_query(query: &str) -> Option<String> {
    let query = query.trim();

    if query.is_empty() {
        return None;
    }

    Some(format!("\"{}\"*", query.replace('"', "\"\"")))
}

pub async fn locate_message(
    archive: &Archive,
    account: &Account,
    message_id: &str,
    page_size: NonZeroU32,
    order: SortOrder,
) -> Result<MessageLocation, ExportReaderError> {
    let account_id = account.id().to_string();
    let row = sqlx::query!(
        r#"
        WITH positioned_messages AS (
          SELECT
            id,
            conversation_id,
            ROW_NUMBER() OVER (
              PARTITION BY conversation_id
              ORDER BY created_at_ms, record_id
            ) - 1 AS oldest_index,
            COUNT(*) OVER (
              PARTITION BY conversation_id
            ) AS total_items
          FROM messages
          WHERE account_id = ?
        )
        SELECT
          conversation_id AS "conversation_id!",
          oldest_index AS "oldest_index!: i64",
          total_items AS "total_items!: i64"
        FROM positioned_messages
        WHERE id = ?
        "#,
        account_id,
        message_id,
    )
    .fetch_optional(archive.pool())
    .await?
    .ok_or_else(|| MessageError::NotFound {
        account_id,
        message_id: message_id.to_owned(),
    })?;

    let oldest_index = row.oldest_index as u64;
    let total_items = row.total_items as u64;
    let page_size = u64::from(page_size.get());
    let page_index = match order {
        SortOrder::OldestFirst => oldest_index / page_size,
        SortOrder::NewestFirst => (total_items - 1 - oldest_index) / page_size,
    };
    let page_index = u32::try_from(page_index).map_err(|_| MessageError::PageIndexOutOfRange {
        message_id: message_id.to_owned(),
    })?;

    Ok(MessageLocation {
        conversation_id: row.conversation_id,
        page_index,
    })
}
