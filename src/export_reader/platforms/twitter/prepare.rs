use std::{fs, path::Path, str::FromStr};

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
    let filename = file_path
        .file_name()
        .ok_or_else(|| ExportReaderError::InvalidExportPath {
            export_file_path: file_path.display().to_string(),
        })?;
    let filename = filename.to_string_lossy();

    let supported_file =
        SupportedFile::from_str(&filename).map_err(|_| ExportReaderError::UnexpectedFilename {
            expected: SupportedFile::DirectMessages.as_ref().to_owned(),
            actual: filename.into_owned(),
        })?;

    match supported_file {
        SupportedFile::DirectMessages => prepare_direct_messages(account_id, file_path),
    }
}

fn prepare_direct_messages(
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
