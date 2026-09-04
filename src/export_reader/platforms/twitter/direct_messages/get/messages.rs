use std::collections::HashMap;

use crate::{
    archive::model::Archive,
    error::ExportReaderError,
    export_reader::{
        account::models::Account,
        platforms::twitter::direct_messages::models::{
            TwitterDMAttachmentsModel, TwitterDMEditHistoryModel, TwitterDMModel,
            TwitterDMReactionsModel,
        },
    },
};

#[derive(Debug, Clone)]
pub struct Reaction {
    event_id: String,
    sender_id: String,
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

#[derive(Debug, Clone)]
pub struct Edit {
    edited_text: String,
    created_at_sec: String,
}

#[derive(Debug, Clone)]
pub struct Attachment {
    source_kind: AttachmentSourceKind,
    source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::Display, strum::AsRefStr)]
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

    let dm_main = sqlx::query_as!(
        TwitterDMModel,
        r#"
      SELECT
          id,
          account_id,
          other_user_id,
          conversation_id,
          message_create_id,
          sender_id,
          recipient_id,
          text AS "text!",
          created_at
      FROM twitter_direct_messages
        WHERE account_id = ?
        AND conversation_id = ?
      ORDER BY created_at, message_create_id
      "#,
        account_id,
        conversation_id
    )
    .fetch_all(archive.pool())
    .await?;

    let dm_attachments = sqlx::query_as!(
        TwitterDMAttachmentsModel,
        r#"
      SELECT
          attachment.main_id,
          attachment.ordinal,
          attachment.source_kind,
          attachment.source
      FROM twitter_dm_attachments AS attachment
      INNER JOIN twitter_direct_messages AS message
          ON message.id = attachment.main_id
      WHERE message.account_id = ?
        AND message.conversation_id = ?
      ORDER BY attachment.main_id, attachment.ordinal
      "#,
        account_id,
        conversation_id
    )
    .fetch_all(archive.pool())
    .await?;

    let dm_reactions = sqlx::query_as!(
        TwitterDMReactionsModel,
        r#"
      SELECT
          reaction.main_id,
          reaction.event_id,
          reaction.sender_id,
          reaction.reaction_key,
          reaction.created_at
      FROM twitter_dm_reactions AS reaction
      INNER JOIN twitter_direct_messages AS message
          ON message.id = reaction.main_id
      WHERE message.account_id = ?
        AND message.conversation_id = ?
      ORDER BY reaction.main_id, reaction.created_at
      "#,
        account_id,
        conversation_id
    )
    .fetch_all(archive.pool())
    .await?;

    let dm_edit_history = sqlx::query_as!(
        TwitterDMEditHistoryModel,
        r#"
      SELECT
          edit.main_id,
          edit.ordinal,
          edit.edited_text,
          edit.created_at_sec
      FROM twitter_dm_edit_history AS edit
      INNER JOIN twitter_direct_messages AS message
          ON message.id = edit.main_id
      WHERE message.account_id = ?
        AND message.conversation_id = ?
      ORDER BY edit.main_id, edit.ordinal
      "#,
        account_id,
        conversation_id
    )
    .fetch_all(archive.pool())
    .await?;

    let mut attachments_by_message: HashMap<String, Vec<Attachment>> = HashMap::new();

    for attachment in dm_attachments {
        let source_kind = attachment.source_kind.parse()?;

        attachments_by_message
            .entry(attachment.main_id)
            .or_default()
            .push(Attachment {
                source_kind,
                source: attachment.source,
            });
    }

    let mut reactions_by_message: HashMap<String, Vec<Reaction>> = HashMap::new();

    for reaction in dm_reactions {
        reactions_by_message
            .entry(reaction.main_id)
            .or_default()
            .push(Reaction {
                event_id: reaction.event_id,
                sender_id: reaction.sender_id,
                reaction_key: ReactionKey::from(reaction.reaction_key.as_str()),
                created_at: reaction.created_at,
            });
    }

    let mut edits_by_message: HashMap<String, Vec<Edit>> = HashMap::new();

    for edit in dm_edit_history {
        edits_by_message
            .entry(edit.main_id)
            .or_default()
            .push(Edit {
                edited_text: edit.edited_text,
                created_at_sec: edit.created_at_sec,
            });
    }
    let mut direct_messages: Vec<DirectMessage> = Vec::with_capacity(dm_main.len());

    for message in dm_main {
        let internal_id = message.id;

        direct_messages.push(DirectMessage {
            id: message.message_create_id,
            conversation_id: message.conversation_id,
            sender_id: message.sender_id,
            recipient_id: message.recipient_id,
            text: message.text,
            created_at: message.created_at,
            attachments: attachments_by_message
                .remove(&internal_id)
                .unwrap_or_default(),
            reactions: reactions_by_message
                .remove(&internal_id)
                .unwrap_or_default(),
            edit_history: edits_by_message.remove(&internal_id).unwrap_or_default(),
        });
    }

    Ok(direct_messages)
}
