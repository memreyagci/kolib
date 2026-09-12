use chrono::DateTime;
use regex::Regex;
use uuid::Uuid;

use crate::{
    error::ExportReaderError,
    export_reader::{
        account::models::AccountId,
        datasets::messages::rows::{
            EditRow, FileAttachmentRow, LinkAttachmentRow, MessageImportRows, MessageRow,
            ReactionRow,
        },
        platforms::twitter::direct_messages::schema::DirectMessagesSchema,
    },
    types::Platform,
};

const TWITTER_DM_MEDIA_MARKER_PREFIX: &str = "https://twitter.com/messages/media/";

enum AttachmentSource {
    File(String),
    Link(String),
}

pub(crate) fn to_rows(
    account_id: &AccountId,
    raw_content: &str,
) -> Result<MessageImportRows, ExportReaderError> {
    let export: DirectMessagesSchema =
        serde_json::from_str::<DirectMessagesSchema>(&js_to_json(raw_content)?)?;

    let mut rows = MessageImportRows::default();

    let account_id = account_id.to_string();
    let platform = Platform::Twitter.to_string();

    for conversation in export {
        let conversation_id = conversation.dm_conversation.conversation_id;

        for message in conversation.dm_conversation.messages {
            let message = message.message_create;
            let main_id = Uuid::now_v7().to_string();
            let record_id = message.id;

            rows.messages.push(MessageRow {
                id: main_id.clone(),
                account_id: account_id.clone(),
                platform: platform.clone(),
                conversation_id: conversation_id.clone(),
                record_id: record_id.clone(),
                sender: message.sender_id,
                recipient: Some(message.recipient_id),
                text: Some(message.text),
                created_at_ms: Some(date_to_unix_timestamp_ms(&message.created_at)?),
            });

            for (ordinal, reaction) in message.reactions.into_iter().enumerate() {
                rows.reactions.push(ReactionRow {
                    main_id: main_id.clone(),
                    ordinal: ordinal as i64,
                    record_id: Some(reaction.event_id),
                    sender: Some(reaction.sender_id),
                    reaction: twitter_reaction_to_emoji(reaction.reaction_key),
                    created_at_ms: Some(date_to_unix_timestamp_ms(&reaction.created_at)?),
                });
            }

            for (ordinal, edit) in message.edit_history.into_iter().enumerate() {
                rows.edits.push(EditRow {
                    main_id: main_id.clone(),
                    ordinal: ordinal as i64,
                    text: edit.edited_text,
                    created_at_ms: Some(edit.created_at_sec.parse::<i64>()? * 1000),
                });
            }

            let mut attachments = Vec::new();

            for url in message.urls {
                // A Twitter media URL's final path component becomes the local
                // archive filename when prefixed with the message record ID.
                if url.expanded == format!("{TWITTER_DM_MEDIA_MARKER_PREFIX}{record_id}") {
                    for media_url in &message.media_urls {
                        let media_url = url::Url::parse(media_url)?;
                        let last_path = media_url
                            .path_segments()
                            .ok_or(ExportReaderError::MediaPathParse)?
                            .next_back()
                            .ok_or(ExportReaderError::MediaPathParse)?;

                        attachments
                            .push(AttachmentSource::File(format!("{record_id}-{last_path}")));
                    }
                } else {
                    attachments.push(AttachmentSource::Link(url.expanded));
                }
            }

            for (ordinal, attachment) in attachments.into_iter().enumerate() {
                match attachment {
                    AttachmentSource::File(file_rel_path) => {
                        rows.file_attachments.push(FileAttachmentRow {
                            main_id: main_id.clone(),
                            ordinal: ordinal as i64,
                            file_rel_path,
                            created_at_ms: None,
                        });
                    }
                    AttachmentSource::Link(url) => {
                        rows.link_attachments.push(LinkAttachmentRow {
                            main_id: main_id.clone(),
                            ordinal: ordinal as i64,
                            url,
                            created_at_ms: None,
                        });
                    }
                }
            }
        }
    }

    Ok(rows)
}

fn date_to_unix_timestamp_ms(date: &str) -> Result<i64, chrono::ParseError> {
    DateTime::parse_from_rfc3339(date).map(|date| date.timestamp_millis())
}

fn twitter_reaction_to_emoji(reaction: String) -> String {
    match reaction.as_str() {
        "agree" => "👍".to_owned(),
        "disagree" => "👎".to_owned(),
        "funny" => "😂".to_owned(),
        "like" => "❤️".to_owned(),
        "sad" => "😔".to_owned(),
        "surprised" => "😮".to_owned(),
        _ => reaction,
    }
}

/// Removes the JavaScript variable declaration in .js files Twitter/X exports.
/// For instance, direct-messages.js starts with:
/// ```javascript
/// window.YTD.direct_messages.part0 =
/// ```
/// When removed, we end up with a JSON array.
fn js_to_json(raw_content: &str) -> Result<String, regex::Error> {
    let re = Regex::new(r"^[^=]*=\s*|;$")?;
    let jsonized = re.replace_all(raw_content.trim(), "");

    Ok(jsonized.to_string())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::to_rows;
    use crate::export_reader::account::models::AccountId;

    const COMPREHENSIVE_EXPORT: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/twitter/direct_messages/comprehensive/direct-messages.js"
    ));

    #[test]
    fn converts_export_to_canonical_message_rows() {
        let account_id = AccountId::new();
        let rows = to_rows(&account_id, COMPREHENSIVE_EXPORT)
            .expect("converting the comprehensive Twitter DM export should succeed");

        assert_eq!(rows.messages.len(), 12);
        assert_eq!(rows.reactions.len(), 5);
        assert_eq!(rows.edits.len(), 5);
        assert_eq!(rows.file_attachments.len(), 5);
        assert_eq!(rows.link_attachments.len(), 2);

        assert!(rows.messages.iter().all(|message| {
            message.account_id == account_id.to_string()
                && message.platform == "twitter"
                && Uuid::parse_str(&message.id).is_ok()
        }));

        let message = rows
            .messages
            .iter()
            .find(|message| message.record_id == "8000000000000000008")
            .expect("the message containing every supported child row should exist");

        assert_eq!(message.created_at_ms, Some(1_788_213_960_008));
        assert_eq!(message.recipient.as_deref(), Some("5555555555555555555"));

        let reactions = rows
            .reactions
            .iter()
            .filter(|reaction| reaction.main_id == message.id)
            .collect::<Vec<_>>();
        assert_eq!(reactions.len(), 2);
        assert_eq!(reactions[0].reaction, "😮");
        assert_eq!(reactions[0].created_at_ms, Some(1_788_214_020_001));
        assert_eq!(reactions[1].reaction, "❤️");

        let edits = rows
            .edits
            .iter()
            .filter(|edit| edit.main_id == message.id)
            .collect::<Vec<_>>();
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].created_at_ms, Some(1_788_214_020_000));

        let file_attachment = rows
            .file_attachments
            .iter()
            .find(|attachment| attachment.main_id == message.id)
            .expect("the message's local attachment should exist");
        assert_eq!(file_attachment.ordinal, 0);
        assert_eq!(
            file_attachment.file_rel_path,
            "8000000000000000008-everything-test-video.mp4"
        );

        let link_attachment = rows
            .link_attachments
            .iter()
            .find(|attachment| attachment.main_id == message.id)
            .expect("the message's link attachment should exist");
        assert_eq!(link_attachment.ordinal, 1);
        assert_eq!(link_attachment.url, "https://youtu.be/dQw4w9WgXcQ");
    }

    #[test]
    fn converts_empty_export_to_empty_rows() {
        let rows = to_rows(&AccountId::new(), "window.YTD.direct_messages.part0 = []")
            .expect("converting an empty Twitter DM export should succeed");

        assert!(rows.messages.is_empty());
        assert!(rows.reactions.is_empty());
        assert!(rows.edits.is_empty());
        assert!(rows.file_attachments.is_empty());
        assert!(rows.link_attachments.is_empty());
    }
}
