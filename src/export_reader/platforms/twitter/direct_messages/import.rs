use std::{
    ffi::OsStr,
    fs::{self},
    path::Path,
};

use crate::{
    archive::model::Archive,
    error::ExportReaderError,
    export_reader::{
        account::models::Account,
        platforms::twitter::direct_messages::{
            FILE_NAME,
            models::{TwitterDMRows, get_rows},
        },
    },
    types::Platform,
};

// TODO: Return a result struct that gives feedback about the import process.
// For instance, how many conversations, messages, etc. inserted.
// Missing media files. and so on.
// TODO: Consider having account_id + dataset_id + platform as identifier
pub async fn import(
    archive: &Archive,
    account: &Account,
    file_path: impl AsRef<Path>,
) -> Result<(), ExportReaderError> {
    if account.platform() != &Platform::Twitter {
        return Err(ExportReaderError::PlatformMismatch {
            acc_platform: account.platform().to_string(),
            importer_platform: Platform::Twitter.to_string(),
        });
    }

    let file_path = file_path.as_ref();

    let filename = file_path
        .file_name()
        .ok_or_else(|| ExportReaderError::InvalidExportPath {
            export_file_path: file_path.display().to_string(),
        })?;
    let content_str = fs::read_to_string(file_path)?;

    if filename != OsStr::new(FILE_NAME) {
        return Err(ExportReaderError::UnexpectedFilename {
            expected: FILE_NAME.to_string(),
            actual: filename.to_string_lossy().into_owned(),
        });
    }

    let to_import: TwitterDMRows = get_rows(account.id(), content_str)?;

    // Create temp dirs and copy the media and raw files there
    // to be moved to the real dir right before transaction commit.
    let tmp_tw_dm_dir = archive
        .folder()
        .join(".tmp/")
        .join("accounts")
        .join(account.id().to_string())
        .join("twitter-direct-messages");

    fs::create_dir_all(&tmp_tw_dm_dir)?;
    fs::create_dir(tmp_tw_dm_dir.join("raw"))?;
    fs::create_dir(tmp_tw_dm_dir.join("media"))?;

    fs::copy(file_path, &tmp_tw_dm_dir.join("raw").join(filename))?;

    // Media files are not a must for import process to be successful.
    // Users not having the media files for any reason should not prevent
    // them from accessing their export files. End-user apps should indicate
    // a missing file within the messages accordingly
    let export_file_dir = &file_path.parent();
    if let Some(efd) = export_file_dir {
        let dm_media_dir = efd.join("direct_messages_media");

        for attachment in to_import
            .attachments
            .iter()
            .filter(|attachment| attachment.source_kind == "file")
        {
            let _ = fs::copy(
                dm_media_dir.join(&attachment.source),
                tmp_tw_dm_dir.join("media").join(&attachment.source),
            );
        }
    }

    let mut tx = archive.pool().begin().await?;

    sqlx::query!(
        "INSERT INTO account_datasets
            (account_id, dataset_type)
            VALUES (?, ?)",
        account.id().to_string(),
        FILE_NAME
    )
    .execute(&mut *tx)
    .await?;

    for main in &to_import.main {
        sqlx::query!(
            "INSERT INTO twitter_direct_messages
                (id, account_id, other_user_id, conversation_id,
                message_create_id, sender_id, recipient_id, text, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &main.id,
            &main.account_id,
            &main.other_user_id,
            &main.conversation_id,
            &main.message_create_id,
            &main.sender_id,
            &main.recipient_id,
            &main.text,
            &main.created_at
        )
        .execute(&mut *tx)
        .await?;
    }

    for reactions in &to_import.reactions {
        sqlx::query!(
            "INSERT INTO twitter_dm_reactions
                (main_id, sender_id, reaction_key, event_id, created_at)
                VALUES (?, ?, ?, ?, ?)",
            &reactions.main_id,
            &reactions.sender_id,
            &reactions.reaction_key,
            &reactions.event_id,
            &reactions.created_at
        )
        .execute(&mut *tx)
        .await?;
    }

    for edits in &to_import.edit_history {
        sqlx::query!(
            "INSERT INTO twitter_dm_edit_history
                (main_id, ordinal, edited_text, created_at_sec)
                VALUES (?, ?, ?, ?)",
            &edits.main_id,
            &edits.ordinal,
            &edits.edited_text,
            &edits.created_at_sec
        )
        .execute(&mut *tx)
        .await?;
    }

    for attachment in &to_import.attachments {
        sqlx::query!(
            "
          INSERT INTO twitter_dm_attachments
              (main_id, ordinal, source_kind, source)
          VALUES (?, ?, ?, ?)
          ",
            &attachment.main_id,
            &attachment.ordinal,
            &attachment.source_kind,
            &attachment.source
        )
        .execute(&mut *tx)
        .await?;
    }

    fs::rename(
        tmp_tw_dm_dir,
        archive
            .folder()
            .join("accounts")
            .join(account.id().to_string()),
    )?;

    tx.commit().await?;

    Ok(())
}
