use crate::{
    archive::model::Archive,
    error::{ExportReaderError, TwitterError},
    export_reader::account::models::Account,
};

use serde_with::serde_as;
use sqlx::types::Json;

#[serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Reaction {
    event_id: String,
    sender_id: String,

    #[serde_as(as = "serde_with::DisplayFromStr")]
    reaction_key: ReactionKey,
    created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, strum::AsRefStr)]
pub enum ReactionKey {
    #[strum(serialize = "agree", to_string = "👍")]
    Agree,

    #[strum(serialize = "disagree", to_string = "👎")]
    Disagree,

    #[strum(to_string = "emoji")]
    Emoji,

    #[strum(serialize = "funny", to_string = "😂")]
    Funny,

    #[strum(serialize = "like", to_string = "❤️")]
    Like,

    #[strum(serialize = "sad", to_string = "😔")]
    Sad,

    #[strum(serialize = "surprised", to_string = "😮")]
    Surprised,

    #[strum(default, transparent)]
    Unknown(String),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Edit {
    edited_text: String,
    created_at_sec: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Attachment {
    source_kind: AttachmentSourceKind,
    source: String,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Deserialize,
    strum::EnumString,
    strum::Display,
    strum::AsRefStr,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AttachmentSourceKind {
    Url,
    File,
}

#[derive(Debug, Clone)]
pub struct DirectMessage {
    id: String,
    conversation_id: String,
    sender_id: String,
    recipient_id: String,
    text: String,
    created_at: String,
    reactions: Vec<Reaction>,
    edit_history: Vec<Edit>,
    attachments: Vec<Attachment>,
}

impl DirectMessage {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn conversation_id(&self) -> &str {
        &self.conversation_id
    }

    pub fn sender_id(&self) -> &str {
        &self.sender_id
    }

    pub fn recipient_id(&self) -> &str {
        &self.recipient_id
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn created_at(&self) -> &str {
        &self.created_at
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

impl Reaction {
    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    pub fn sender_id(&self) -> &str {
        &self.sender_id
    }

    pub fn reaction_key(&self) -> &str {
        self.reaction_key.as_ref()
    }

    pub fn created_at(&self) -> &str {
        &self.created_at
    }
}

impl Edit {
    pub fn edited_text(&self) -> &str {
        &self.edited_text
    }

    pub fn created_at_sec(&self) -> &str {
        &self.created_at_sec
    }
}

impl Attachment {
    pub fn source_kind(&self) -> AttachmentSourceKind {
        self.source_kind
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

pub async fn get_messages_by_conversation(
    archive: &Archive,
    account: &Account,
    conversation_id: &str,
) -> Result<Vec<DirectMessage>, ExportReaderError> {
    let account_id = account.id().to_string();

    let rows = sqlx::query!(
        r#"
  SELECT
    message.message_create_id AS "id!",
    message.conversation_id,
    message.sender_id,
    message.recipient_id,
    message.text AS "text!",
    message.created_at,
    (
      SELECT json_group_array(
        json_object(
          'event_id', reaction.event_id,
          'sender_id', reaction.sender_id,
          'reaction_key', reaction.reaction_key,
          'created_at', reaction.created_at
        )
      )
      FROM (
        SELECT
          event_id,
          sender_id,
          reaction_key,
          created_at
        FROM twitter_dm_reactions
        WHERE main_id = message.id
        ORDER BY created_at, event_id
      ) AS reaction
    ) AS "reactions!: Json<Vec<Reaction>>",
    (
      SELECT json_group_array(
        json_object(
          'edited_text', edit.edited_text,
          'created_at_sec', edit.created_at_sec
        )
      )
      FROM (
        SELECT
          edited_text,
          created_at_sec
        FROM twitter_dm_edit_history
        WHERE main_id = message.id
        ORDER BY ordinal
      ) AS edit
    ) AS "edit_history!: Json<Vec<Edit>>",
    (
      SELECT json_group_array(
        json_object(
          'source_kind', attachment.source_kind,
          'source', attachment.source
        )
      )
      FROM (
        SELECT
          source_kind,
          source
        FROM twitter_dm_attachments
        WHERE main_id = message.id
        ORDER BY ordinal
      ) AS attachment
    ) AS "attachments!: Json<Vec<Attachment>>"
  FROM twitter_direct_messages AS message
  WHERE message.account_id = ?
    AND message.conversation_id = ?
  ORDER BY message.created_at, message.message_create_id
  "#,
        account_id,
        conversation_id
    )
    .fetch_all(archive.pool())
    .await?;

    if rows.is_empty() {
        return Err(TwitterError::ConversationNotFound {
            account_id,
            conversation_id: conversation_id.to_owned(),
        }
        .into());
    }

    let direct_messages = rows
        .into_iter()
        .map(|row| DirectMessage {
            id: row.id,
            conversation_id: row.conversation_id,
            sender_id: row.sender_id,
            recipient_id: row.recipient_id,
            text: row.text,
            created_at: row.created_at,
            reactions: row.reactions.0,
            edit_history: row.edit_history.0,
            attachments: row.attachments.0,
        })
        .collect();

    Ok(direct_messages)
}
