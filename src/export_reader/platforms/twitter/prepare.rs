use std::{path::Path, str::FromStr};

use crate::{
    error::ExportReaderError,
    export_reader::{
        account::models::AccountId,
        import::PreparedImport,
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
        SupportedFile::DirectMessages => direct_messages::prepare_import(account_id, file_path),
    }
}
