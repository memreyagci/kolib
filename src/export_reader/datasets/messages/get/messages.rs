use std::{path::PathBuf, str::FromStr};

use sqlx::types::Json;

use crate::{
    archive::model::Archive,
    error::{ExportReaderError, MessageError},
    export_reader::{
        account::models::Account,
        datasets::DatasetType,
        pagination::{Page, PageRequest, SortOrder},
    },
    types::{Platform, Timestamp},
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Reaction {
    record_id: Option<String>,
    sender: Option<String>,
    reaction: String,
    created_at: Option<Timestamp>,
}

impl Reaction {
    pub fn record_id(&self) -> Option<&str> {
        self.record_id.as_deref()
    }

    pub fn sender(&self) -> Option<&str> {
        self.sender.as_deref()
    }

    pub fn reaction(&self) -> &str {
        &self.reaction
    }

    pub fn created_at(&self) -> Option<Timestamp> {
        self.created_at
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Edit {
    text: String,
    created_at: Option<Timestamp>,
}

impl Edit {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn created_at(&self) -> Option<Timestamp> {
        self.created_at
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Attachment {
    source_kind: AttachmentSourceKind,
    source: String,
    created_at: Option<Timestamp>,
}

impl Attachment {
    pub fn source_kind(&self) -> AttachmentSourceKind {
        self.source_kind
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn full_path(&self, archive: &Archive, account: &Account) -> Option<PathBuf> {
        if self.source_kind != AttachmentSourceKind::File {
            return None;
        }

        Some(
            archive
                .dataset_directory(account, DatasetType::Messages)
                .join("media")
                .join(&self.source),
        )
    }

    pub fn created_at(&self) -> Option<Timestamp> {
        self.created_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentSourceKind {
    File,
    Url,
}

#[derive(Debug, Clone)]
pub struct Message {
    id: String,
    platform: Platform,
    conversation_id: String,
    record_id: String,
    sender: String,
    recipient: Option<String>,
    text: Option<String>,
    created_at: Option<Timestamp>,
    reactions: Vec<Reaction>,
    edit_history: Vec<Edit>,
    attachments: Vec<Attachment>,
}

impl Message {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn conversation_id(&self) -> &str {
        &self.conversation_id
    }

    pub fn record_id(&self) -> &str {
        &self.record_id
    }

    pub fn sender(&self) -> &str {
        &self.sender
    }

    pub fn recipient(&self) -> Option<&str> {
        self.recipient.as_deref()
    }

    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    pub fn created_at(&self) -> Option<Timestamp> {
        self.created_at
    }

    pub fn reactions(&self) -> &[Reaction] {
        &self.reactions
    }

    pub fn edit_history(&self) -> &[Edit] {
        &self.edit_history
    }

    pub fn attachments(&self) -> &[Attachment] {
        &self.attachments
    }
}

#[derive(sqlx::FromRow)]
struct MessageQueryRow {
    id: String,
    platform: String,
    conversation_id: String,
    record_id: String,
    sender: String,
    recipient: Option<String>,
    text: Option<String>,
    created_at_ms: Option<i64>,
    reactions: Json<Vec<Reaction>>,
    edit_history: Json<Vec<Edit>>,
    attachments: Json<Vec<Attachment>>,
}

pub async fn get_messages_by_conversation(
    archive: &Archive,
    account: &Account,
    conversation_id: &str,
) -> Result<Vec<Message>, ExportReaderError> {
    let account_id = account.id().to_string();
    let messages = fetch_messages(archive, account, conversation_id, i64::MAX, 0).await?;

    if messages.is_empty() {
        return Err(MessageError::ConversationNotFound {
            account_id,
            conversation_id: conversation_id.to_owned(),
        }
        .into());
    }

    Ok(messages)
}

async fn fetch_messages(
    archive: &Archive,
    account: &Account,
    conversation_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Message>, ExportReaderError> {
    let account_id = account.id().to_string();

    let rows = sqlx::query_as::<_, MessageQueryRow>(
        r#"
        SELECT
          message.id,
          message.platform,
          message.conversation_id,
          message.record_id,
          message.sender,
          message.recipient,
          message.text,
          message.created_at_ms,
          (
            SELECT json_group_array(
              json_object(
                'record_id', reaction.record_id,
                'sender', reaction.sender,
                'reaction', reaction.reaction,
                'created_at', reaction.created_at_ms
              )
            )
            FROM (
              SELECT
                record_id,
                sender,
                reaction,
                created_at_ms
              FROM message_reactions
              WHERE main_id = message.id
              ORDER BY ordinal
            ) AS reaction
          ) AS reactions,
          (
            SELECT json_group_array(
              json_object(
                'text', edit.text,
                'created_at', edit.created_at_ms
              )
            )
            FROM (
              SELECT
                text,
                created_at_ms
              FROM message_edits
              WHERE main_id = message.id
              ORDER BY ordinal
            ) AS edit
          ) AS edit_history,
          (
            SELECT json_group_array(
              json_object(
                'source_kind', attachment.source_kind,
                'source', attachment.source,
                'created_at', attachment.created_at_ms
              )
            )
            FROM (
              SELECT
                ordinal,
                'file' AS source_kind,
                filename AS source,
                created_at_ms
              FROM message_file_attachments
              WHERE main_id = message.id

              UNION ALL

              SELECT
                ordinal,
                'url' AS source_kind,
                url AS source,
                created_at_ms
              FROM message_link_attachments
              WHERE main_id = message.id

              ORDER BY ordinal, source_kind
            ) AS attachment
          ) AS attachments
        FROM messages AS message
        WHERE message.account_id = ?
          AND message.conversation_id = ?
        ORDER BY
          message.created_at_ms,
          message.record_id,
          message.id
        LIMIT ?
        OFFSET ?
        "#,
    )
    .bind(account_id)
    .bind(conversation_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(archive.pool())
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(Message {
                id: row.id,
                platform: Platform::from_str(&row.platform)?,
                conversation_id: row.conversation_id,
                record_id: row.record_id,
                sender: row.sender,
                recipient: row.recipient,
                text: row.text,
                created_at: row.created_at_ms.map(Timestamp::from),
                reactions: row.reactions.0,
                edit_history: row.edit_history.0,
                attachments: row.attachments.0,
            })
        })
        .collect()
}

pub async fn get_message_page_by_conversation(
    archive: &Archive,
    account: &Account,
    conversation_id: &str,
    pagination: PageRequest,
) -> Result<Page<Message>, ExportReaderError> {
    let account_id = account.id().to_string();

    let total_items = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM messages
        WHERE account_id = ?
          AND conversation_id = ?
        "#,
    )
    .bind(&account_id)
    .bind(conversation_id)
    .fetch_one(archive.pool())
    .await?;

    if total_items == 0 {
        return Err(MessageError::ConversationNotFound {
            account_id,
            conversation_id: conversation_id.to_owned(),
        }
        .into());
    }

    let total_items = total_items as u64;
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

    let mut messages = fetch_messages(
        archive,
        account,
        conversation_id,
        limit as i64,
        offset as i64,
    )
    .await?;

    if pagination.order() == SortOrder::NewestFirst {
        messages.reverse();
    }

    Ok(Page::new(messages, pagination, total_items))
}
