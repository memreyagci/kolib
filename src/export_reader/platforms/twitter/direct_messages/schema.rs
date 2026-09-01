//! Schema of "direct-messages.js" file.
//! Created using quicktype: https://quicktype.io/
//! and modifications made as needed.

use serde::{Deserialize, Serialize};

pub type DirectMessagesSchema = Vec<DirectMessagesElement>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectMessagesElement {
    pub dm_conversation: DmConversation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DmConversation {
    pub conversation_id: String,

    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub message_create: MessageCreate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde_with::serde_as]
#[serde(rename_all = "camelCase")]
pub struct MessageCreate {
    pub recipient_id: String,

    pub reactions: Vec<Reaction>,

    pub urls: Vec<Url>,

    pub text: String,

    pub media_urls: Vec<String>,

    pub sender_id: String,

    pub id: String,

    pub created_at: String,

    #[serde(default)]
    pub edit_history: Vec<EditHistory>,

    // "edited: Option<bool>" is unnecessary, as its appearance is inconsistent, and edit_history already
    // gives the info on whether the message is edited and more.
    #[serde(flatten)]
    pub extra_stuff: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde_with::serde_as]
#[serde(rename_all = "camelCase")]
pub struct EditHistory {
    pub edited_text: String,

    pub created_at_sec: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde_with::serde_as]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    pub sender_id: String,

    pub reaction_key: String,

    pub event_id: String,

    pub created_at: String,
}

// TODO: Consider finding a way to convert keywords to emojis without needing to run to_string() function
#[derive(Debug, Clone, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum ReactionKey {
    #[strum(to_string = "👍")]
    Agree,

    #[strum(to_string = "👎")]
    Disagree,

    // Some "reaction_key"s are represented with "emoji" keyword, so it is not possible to know
    // which emoji was actually sent.
    Emoji,

    #[strum(to_string = "😂")]
    Funny,

    #[strum(to_string = "❤️")]
    Like,

    #[strum(to_string = "😔")]
    Sad,

    #[strum(to_string = "😮")]
    Surprised,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Url {
    pub expanded: String,

    // "url: String" is the shortened "t.co" link of Twitter/x
    // "display: String" is the truncated version shown in messages
    // thus, display is all we need
    #[serde(flatten)]
    pub extra_stuff: serde_json::Value,
}
