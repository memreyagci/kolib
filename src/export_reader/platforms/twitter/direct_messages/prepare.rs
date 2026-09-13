use std::{fs, path::Path};

use crate::{
    error::ExportReaderError,
    export_reader::{
        account::models::AccountId,
        import::{DatasetRows, PreparedImport, PreparedMediaFile},
        platforms::twitter::{direct_messages, supported_files::SupportedFile},
    },
};

pub(crate) fn prepare_import(
    account_id: &AccountId,
    file_path: &Path,
) -> Result<PreparedImport, ExportReaderError> {
    let supported_file = SupportedFile::DirectMessages;
    let raw_content = fs::read_to_string(file_path)?;
    let rows = direct_messages::to_rows::to_rows(account_id, &raw_content)?;

    let media_files = supported_file
        .media_directory()
        .map(|directory| {
            let directory = file_path
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .join(directory);

            rows.file_attachments
                .iter()
                .map(|attachment| PreparedMediaFile {
                    source_path: directory.join(&attachment.filename),
                    filename: attachment.filename.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(PreparedImport {
        dataset: DatasetRows::Messages(rows),
        source_file: file_path.to_owned(),
        media_files,
    })
}
