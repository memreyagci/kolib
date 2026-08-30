use crate::{
    archive::model::Archive,
    error::ExportReaderError,
    export_reader::{
        account::models::Account,
        platforms::twitter::direct_messages::models::{
            TwitterDMAttachmentsModel, TwitterDMEditHistoryModel, TwitterDMModel,
            TwitterDMReactionsModel, TwitterDMRows,
        },
    },
};

// TODO: TwitterDMRows result requires user to figure out which reactions, attachments,
// and/or edits to belong to which message. Instead, return a vector of something like this:
// ```
// pub struct DirectMessage {
//       pub id: String,
//       pub sender_id: String,
//       pub recipient_id: String,
//       pub text: String,
//       pub created_at: String,
//       pub reactions: Vec<Reaction>,
//       pub edit_history: Vec<Edit>,
//       pub attachments: Vec<Attachment>,
//   }
// ```
// which is a true representative of a single message.
pub async fn get(archive: &Archive, account: &Account) -> Result<TwitterDMRows, ExportReaderError> {
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
      "#,
        account_id
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
      ORDER BY attachment.main_id, attachment.ordinal
      "#,
        account_id
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
      ORDER BY reaction.main_id, reaction.created_at
      "#,
        &account_id
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
      ORDER BY edit.main_id, edit.ordinal
      "#,
        &account_id
    )
    .fetch_all(archive.pool())
    .await?;

    Ok(TwitterDMRows {
        main: dm_main,
        reactions: dm_reactions,
        edit_history: dm_edit_history,
        attachments: dm_attachments,
    })
}
