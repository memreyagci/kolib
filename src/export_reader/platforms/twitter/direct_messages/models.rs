use chrono::DateTime;
use regex::Regex;
use uuid::Uuid;

use crate::export_reader::platforms::twitter::direct_messages::schema::DirectMessagesSchema;

const TWITTER_DM_MEDIA_MARKER_PREFIX: &str = "https://twitter.com/messages/media/";

#[derive(sqlx::FromRow, Debug)]
pub struct TwitterDMModel {
    pub id: String,
    pub account_id: String,
    pub other_user_id: String,
    pub conversation_id: String,
    pub message_create_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub text: String,
    pub created_at: i64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct TwitterDMReactionsModel {
    pub sender_id: String,
    pub reaction_key: String,
    pub event_id: String,
    pub created_at: i64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct TwitterDMEditHistoryModel {
    pub edited_text: String,
    pub created_at_sec: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct TwitterDMAttachmentsModel {
    pub id: String,
    pub message_id: String,
    pub ordinal: u8,
    pub external: u8,
    pub target: String,
}

#[derive(Debug)]
pub struct TwitterDMRows {
    pub main: Vec<TwitterDMModel>,
    pub reactions: Vec<TwitterDMReactionsModel>,
    pub edit_history: Vec<TwitterDMEditHistoryModel>,
    pub attachments: Vec<TwitterDMAttachmentsModel>,
}

pub(crate) fn get_rows(account_id: Uuid, content_raw: String) -> TwitterDMRows {
    let content_json: DirectMessagesSchema =
        serde_json::from_str::<DirectMessagesSchema>(&js_to_json(content_raw)).unwrap();

    let mut dm_main: Vec<TwitterDMModel> = Vec::new();
    let mut dm_reactions: Vec<TwitterDMReactionsModel> = Vec::new();
    let mut dm_edit_history: Vec<TwitterDMEditHistoryModel> = Vec::new();
    let mut dm_attachments: Vec<TwitterDMAttachmentsModel> = Vec::new();

    for c in content_json {
        for message in c.dm_conversation.messages {
            dm_main.push(TwitterDMModel {
                id: Uuid::now_v7().to_string(),
                account_id: account_id.to_string(),
                other_user_id: "".to_string(), // TODO: Do migration to remove this or make it nullable
                conversation_id: c.dm_conversation.conversation_id.to_owned(),
                message_create_id: message.message_create.id.to_owned(),
                sender_id: message.message_create.sender_id,
                recipient_id: message.message_create.recipient_id,
                text: message.message_create.text,
                created_at: date_to_unix_time_stamp(&message.message_create.created_at).unwrap(),
            });

            for reaction in message.message_create.reactions {
                dm_reactions.push(TwitterDMReactionsModel {
                    sender_id: reaction.sender_id,
                    reaction_key: reaction.reaction_key.to_string(),
                    event_id: reaction.event_id,
                    created_at: date_to_unix_time_stamp(&message.message_create.created_at)
                        .unwrap(),
                });
            }

            if let Some(edit_history) = message.message_create.edit_history {
                for e in edit_history {
                    dm_edit_history.push(TwitterDMEditHistoryModel {
                        edited_text: e.edited_text,
                        created_at_sec: e.created_at_sec,
                    });
                }
            }

            for url in message.message_create.urls {
                // Some messages have two urls/media files attached, and keeping the order is
                // preferred, since there might images that only make when together and in their order.
                let mut ordinal = 0;

                // Consider having urls and attachments as 2 different tables.
                if url.expanded
                    == format!(
                        "{TWITTER_DM_MEDIA_MARKER_PREFIX}{}",
                        message.message_create.id.to_owned()
                    )
                {
                    for media_url in message.message_create.media_urls.to_owned() {
                        // file name as appears in twitter export files. It is the last part of the
                        // media_url
                        let media_file_name = url::Url::parse(&media_url)
                            .unwrap()
                            .path_segments()
                            .unwrap()
                            .last()
                            .unwrap()
                            .to_string();

                        dm_attachments.push(TwitterDMAttachmentsModel {
                            id: Uuid::now_v7().to_string(),
                            message_id: message.message_create.id.to_owned(),
                            ordinal: ordinal,
                            external: 0,
                            target: media_file_name,
                        });
                        ordinal += 1;
                    }
                } else {
                    dm_attachments.push(TwitterDMAttachmentsModel {
                        id: Uuid::now_v7().to_string(),
                        message_id: message.message_create.id.to_owned(),
                        ordinal,
                        external: 1,
                        target: url.expanded,
                    });
                }
            }
        }
    }

    TwitterDMRows {
        main: dm_main,
        reactions: dm_reactions,
        edit_history: dm_edit_history,
        attachments: dm_attachments,
    }
}

fn date_to_unix_time_stamp(date: &str) -> Result<i64, ()> {
    Ok(DateTime::parse_from_rfc3339(&date)
        .map(|dt| dt.timestamp())
        .unwrap_or(0))
}

fn js_to_json(raw_content: String) -> String {
    let re = Regex::new(r"^[^=]*=\s*|;$").unwrap();
    let jsonized = re.replace_all(raw_content.trim(), "");

    jsonized.to_string()
}
