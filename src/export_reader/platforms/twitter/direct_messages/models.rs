use chrono::DateTime;
use regex::Regex;
use uuid::Uuid;

use crate::export_reader::platforms::twitter::direct_messages::schema::DirectMessagesSchema;

const TWITTER_DM_MEDIA_MARKER_PREFIX: &str = "https://twitter.com/messages/media/";

#[derive(sqlx::FromRow, Debug)]
pub struct TwitterDMModel {
    pub(crate) id: String,
    pub(crate) account_id: String,
    pub(crate) other_user_id: String,
    pub(crate) conversation_id: String,
    pub(crate) message_create_id: String,
    pub(crate) sender_id: String,
    pub(crate) recipient_id: String,
    pub(crate) text: String,
    pub(crate) created_at: i64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct TwitterDMReactionsModel {
    pub(crate) id: String,
    pub(crate) main_id: String,
    pub(crate) sender_id: String,
    pub(crate) reaction_key: String,
    pub(crate) event_id: String,
    pub(crate) created_at: i64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct TwitterDMEditHistoryModel {
    pub(crate) id: String,
    pub(crate) main_id: String,
    pub(crate) edited_text: String,
    pub(crate) created_at_sec: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct TwitterDMAttachmentsModel {
    pub(crate) id: String,
    pub(crate) main_id: String,
    pub(crate) external: u8,
    pub(crate) target: String,
}

#[derive(Debug)]
pub struct TwitterDMRows {
    pub(crate) main: Vec<TwitterDMModel>,
    pub(crate) reactions: Vec<TwitterDMReactionsModel>,
    pub(crate) edit_history: Vec<TwitterDMEditHistoryModel>,
    pub(crate) attachments: Vec<TwitterDMAttachmentsModel>,
}

pub(crate) fn get_rows(
    account_id: Uuid,
    content_raw: String,
) -> Result<TwitterDMRows, Box<dyn std::error::Error>> {
    let content_json: DirectMessagesSchema =
        serde_json::from_str::<DirectMessagesSchema>(&js_to_json(content_raw)?)?;

    let mut dm_main: Vec<TwitterDMModel> = Vec::new();
    let mut dm_reactions: Vec<TwitterDMReactionsModel> = Vec::new();
    let mut dm_edit_history: Vec<TwitterDMEditHistoryModel> = Vec::new();
    let mut dm_attachments: Vec<TwitterDMAttachmentsModel> = Vec::new();

    for c in content_json {
        for message in c.dm_conversation.messages {
            let message_id = Uuid::now_v7().to_string();
            dm_main.push(TwitterDMModel {
                id: message_id.to_owned(),
                account_id: account_id.to_string(),
                other_user_id: "".to_string(), // TODO: Do migration to remove this or make it nullable
                conversation_id: c.dm_conversation.conversation_id.to_owned(),
                message_create_id: message.message_create.id.to_owned(),
                sender_id: message.message_create.sender_id,
                recipient_id: message.message_create.recipient_id,
                text: message.message_create.text,
                created_at: date_to_unix_time_stamp(&message.message_create.created_at)?,
            });

            for reaction in message.message_create.reactions {
                dm_reactions.push(TwitterDMReactionsModel {
                    id: Uuid::now_v7().to_string(),
                    main_id: message_id.to_owned(), // not message_create.id, but the pk of the main table
                    sender_id: reaction.sender_id,
                    reaction_key: reaction.reaction_key.to_string(),
                    event_id: reaction.event_id,
                    created_at: date_to_unix_time_stamp(&message.message_create.created_at)?,
                });
            }

            if let Some(edit_history) = message.message_create.edit_history {
                for e in edit_history {
                    dm_edit_history.push(TwitterDMEditHistoryModel {
                        id: Uuid::now_v7().to_string(),
                        main_id: message_id.to_owned(),
                        edited_text: e.edited_text,
                        created_at_sec: e.created_at_sec,
                    });
                }
            }

            for url in message.message_create.urls {
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
                            main_id: message_id.to_owned(),
                            external: 0,
                            target: media_file_name,
                        });
                    }
                } else {
                    dm_attachments.push(TwitterDMAttachmentsModel {
                        id: Uuid::now_v7().to_string(),
                        main_id: message_id.to_owned(),
                        external: 1,
                        target: url.expanded,
                    });
                }
            }
        }
    }

    Ok(TwitterDMRows {
        main: dm_main,
        reactions: dm_reactions,
        edit_history: dm_edit_history,
        attachments: dm_attachments,
    })
}

fn date_to_unix_time_stamp(date: &str) -> Result<i64, chrono::ParseError> {
    Ok(DateTime::parse_from_rfc3339(&date).map(|dt| dt.timestamp())?)
}

/// Removes the JavaScript variable declaration in .js files Twitter/X exports.
/// For instance, direct-messages.js starts with:
/// ```javascript
/// window.YTD.direct_messages.part0 =
/// ```
/// When removed, we end up with a JSON array.
fn js_to_json(raw_content: String) -> Result<String, regex::Error> {
    let re = Regex::new(r"^[^=]*=\s*|;$")?;
    let jsonized = re.replace_all(raw_content.trim(), "");

    Ok(jsonized.to_string())
}
