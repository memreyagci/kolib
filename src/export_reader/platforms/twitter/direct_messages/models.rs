use chrono::DateTime;
use regex::Regex;
use uuid::Uuid;

use crate::export_reader::platforms::twitter::direct_messages::schema::{
    DirectMessagesSchema, EditHistory, Reaction,
};

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct TwitterDirectMessagesModel {
    pub id: String,
    pub account_id: String,
    pub other_user_id: String,
    pub conversation_id: String,
    pub message_create_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub text: String,
    pub created_at: i64,
    // TODO: Turn reactions and edit_history into different tables
    pub reactions: Vec<Reaction>,
    pub edit_history: Option<Vec<EditHistory>>,
}

impl TwitterDirectMessagesModel {
    pub(crate) fn new(account_id: Uuid, content_raw: String) -> Vec<Self> {
        let content_json: DirectMessagesSchema =
            serde_json::from_str::<DirectMessagesSchema>(&Self::js_to_json(content_raw)).unwrap();

        let mut rows: Vec<Self> = Vec::new();

        for c in content_json {
            for m in c.dm_conversation.messages {
                rows.push(TwitterDirectMessagesModel {
                    id: Uuid::now_v7().to_string(),
                    account_id: account_id.to_string(),
                    other_user_id: "".to_string(), // TODO: Do migration to remove this or make it nullable
                    conversation_id: c.dm_conversation.conversation_id.to_owned(),
                    message_create_id: m.message_create.id,
                    sender_id: m.message_create.sender_id,
                    recipient_id: m.message_create.recipient_id,
                    text: m.message_create.text,
                    created_at: DateTime::parse_from_rfc3339(&m.message_create.created_at)
                        .map(|dt| dt.timestamp())
                        .unwrap_or(0),
                    reactions: m.message_create.reactions,
                    edit_history: m.message_create.edit_history,
                });
            }
        }

        rows
    }

    fn js_to_json(raw_content: String) -> String {
        let re = Regex::new(r"^[^=]*=\s*|;$").unwrap();
        let jsonized = re.replace_all(raw_content.trim(), "");

        jsonized.to_string()
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use uuid::Uuid;

    use crate::{
        export_reader::{
            account::models::Account,
            platforms::twitter::direct_messages::models::TwitterDirectMessagesModel,
        },
        types::Platform,
    };

    #[test]
    fn test_model_creation() {
        let acc = AccountModel::new("test".to_string(), Platform::Twitter);
        let cont = fs::read_to_string(
            "/Users/emre/Documents/repos/koli-server/samples_real/dm_twitter.json",
        )
        .unwrap();
        let tdm = TwitterDirectMessagesModel::new(acc.id(), cont);

        println!("{tdm:?}");
    }
}
