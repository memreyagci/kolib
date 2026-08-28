use chrono::DateTime;
use regex::Regex;
use uuid::Uuid;

use crate::{
    error::ExportReaderError,
    export_reader::platforms::twitter::direct_messages::schema::DirectMessagesSchema,
};

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
    pub(crate) main_id: String,
    pub(crate) event_id: String,
    pub(crate) sender_id: String,
    pub(crate) reaction_key: String,
    pub(crate) created_at: i64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct TwitterDMEditHistoryModel {
    pub(crate) main_id: String,
    pub(crate) ordinal: i64,
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
) -> Result<TwitterDMRows, ExportReaderError> {
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
                    main_id: message_id.to_owned(), // not message_create.id, but the pk of the main table
                    event_id: reaction.event_id,
                    sender_id: reaction.sender_id,
                    reaction_key: reaction.reaction_key,
                    created_at: date_to_unix_time_stamp(&reaction.created_at)?,
                });
            }

            if let Some(edit_history) = message.message_create.edit_history {
                for (ordinal, edit) in edit_history.into_iter().enumerate() {
                    dm_edit_history.push(TwitterDMEditHistoryModel {
                        main_id: message_id.to_owned(),
                        ordinal: ordinal as i64,
                        edited_text: edit.edited_text,
                        created_at_sec: edit.created_at_sec,
                    });
                }
            }

            // TODO: Consider having URLs and attachments as 2 different tables.
            for url in message.message_create.urls {
                // If "urls.expanded" starts with "https://twitter.com/messages/media/",
                // that indicates it is a media attachment that can be found in direct_messages_media/
                // directory, which is at the same path with other export files.
                //
                // It is not possible to derive the filename from "urls.expanded".
                // However, it is with mediaUrls.
                //
                // Three types of mediaUrls that represent a local file in the archive:
                //
                // https://video.twimg.com/dm_gif/123/IdsidssISAONdwqio92Ie1n29djsal-DSijd392dA.mp4
                // Filename would be: "123-IdsidssISAONdwqio92Ie1n29djsal-DSijd392dA.mp4"
                //
                // https://video.twimg.com/dm_video/123/vid/avc1/720x1066/IdsidssISAONdwqio92Ie1n29djsal-DSijd392dA.mp4?tag=1
                // Filename would be: "123-IdsidssISAONdwqio92Ie1n29djsal-DSijd392dA.mp4"
                //
                // https://ton.twitter.com/dm/123/456/ABcD1E2g.jpg
                // Filename would be: "123-ABcD1E2g.jpg"
                //
                // "123" is message_create.id. Thus, what we need is to merge message_create.id
                // with the last path of the mediaUrl, and strip any query strings, if exists, such as
                // "?tag=1" in the above example, and concatenate those two with a "-" between them.
                if url.expanded
                    == format!(
                        "{TWITTER_DM_MEDIA_MARKER_PREFIX}{}",
                        message.message_create.id.to_owned()
                    )
                {
                    for media_url in message.message_create.media_urls.to_owned() {
                        let media_url_parsed = url::Url::parse(&media_url)?;
                        let last_path = media_url_parsed
                            .path_segments()
                            .ok_or(ExportReaderError::MediaPathParseError)?
                            .next_back()
                            .ok_or(ExportReaderError::MediaPathParseError)?;
                        let media_file_name =
                            format!("{}-{}", message.message_create.id, last_path);

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
