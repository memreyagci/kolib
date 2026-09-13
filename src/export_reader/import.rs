use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::{
    archive::model::Archive,
    error::ExportReaderError,
    export_reader::{
        account::models::Account,
        datasets::{DatasetType, messages, messages::rows::MessageImportRows},
        platforms,
    },
    types::Platform,
};

pub(crate) enum DatasetRows {
    Messages(MessageImportRows),
}

impl DatasetRows {
    fn is_empty(&self) -> bool {
        match self {
            Self::Messages(rows) => rows.messages.is_empty(),
        }
    }

    fn dataset_type(&self) -> DatasetType {
        match self {
            Self::Messages(_) => DatasetType::Messages,
        }
    }
}

pub(crate) struct PreparedMediaFile {
    pub(crate) source_path: PathBuf,
    pub(crate) filename: String,
}

pub(crate) struct PreparedImport {
    pub(crate) dataset: DatasetRows,
    pub(crate) source_file: PathBuf,
    pub(crate) media_files: Vec<PreparedMediaFile>,
}

/// This function is used to import an export file to the given account that is in the given archive.
/// It automatically detects dataset type, copies necessary files, and returns a result accordingly.
pub async fn import(
    archive: &Archive,
    account: &Account,
    file_path: impl AsRef<Path>,
) -> Result<(), ExportReaderError> {
    let file_path = file_path.as_ref();

    // Matches account platform to their prepare_import function, which returns the rows the import
    // the source file, and media files paths to be copied in PreparedImport.
    let prepared: PreparedImport = match account.platform() {
        Platform::Twitter => platforms::twitter::prepare_import(account.id(), file_path)?,
        platform => {
            return Err(ExportReaderError::UnsupportedPlatform {
                platform: platform.to_string(),
            });
        }
    };

    if prepared.dataset.is_empty() {
        return Ok(());
    }

    // Initially, copy the media files to a .tmp dir inside archive, so it in case of database
    // failure, they are removed. Otherwise, they are instantly moved.
    let staging_root = archive
        .folder()
        .join(".tmp")
        .join(Uuid::now_v7().to_string());
    let staged_dataset_directory = staging_root.join("dataset");

    if let Err(error) = stage_files(&prepared, &staged_dataset_directory) {
        let _ = fs::remove_dir_all(&staging_root);
        return Err(error);
    }

    let mut transaction = match archive.pool().begin().await {
        Ok(transaction) => transaction,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_root);
            return Err(error.into());
        }
    };

    let insert_result = match &prepared.dataset {
        DatasetRows::Messages(rows) => messages::insert(&mut transaction, rows).await,
    };

    if let Err(error) = insert_result {
        let _ = transaction.rollback().await;
        let _ = fs::remove_dir_all(&staging_root);
        return Err(error);
    }

    // In case a manual deletion of accounts and <account-id> dir occured.
    fs::create_dir_all(archive.account_directory(account))?;
    let archive_dataset_directory =
        archive.dataset_directory(account, prepared.dataset.dataset_type());

    if let Err(error) = fs::rename(&staged_dataset_directory, &archive_dataset_directory) {
        let _ = transaction.rollback().await;
        let _ = fs::remove_dir_all(&staging_root);
        return Err(error.into());
    }

    if let Err(error) = transaction.commit().await {
        let _ = fs::remove_dir_all(&archive_dataset_directory);
        return Err(error.into());
    }

    let _ = fs::remove_dir(&staging_root);

    Ok(())
}

// Copies raw file and media files to a .tmp/ dir in archive.
fn stage_files(
    prepared: &PreparedImport,
    staged_dataset_directory: &Path,
) -> Result<(), ExportReaderError> {
    let raw_directory = staged_dataset_directory.join("raw");
    let media_directory = staged_dataset_directory.join("media");

    fs::create_dir_all(&raw_directory)?;
    fs::create_dir(&media_directory)?;

    let source_filename =
        prepared
            .source_file
            .file_name()
            .ok_or_else(|| ExportReaderError::InvalidExportPath {
                export_file_path: prepared.source_file.display().to_string(),
            })?;

    fs::copy(&prepared.source_file, raw_directory.join(source_filename))?;

    for media_file in &prepared.media_files {
        validate_filename(&media_file.filename)?;

        match fs::copy(
            &media_file.source_path,
            media_directory.join(&media_file.filename),
        ) {
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

fn validate_filename(filename: &str) -> Result<(), ExportReaderError> {
    let path = Path::new(filename);

    if path.file_name() != Some(path.as_os_str()) {
        return Err(ExportReaderError::InvalidMediaFilename {
            filename: filename.to_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_filename;

    #[test]
    fn accepts_plain_media_filename() {
        assert!(validate_filename("message-id-video.mp4").is_ok());
    }

    #[test]
    fn rejects_media_path_as_filename() {
        assert!(validate_filename("../outside.mp4").is_err());
        assert!(validate_filename("nested/video.mp4").is_err());
        assert!(validate_filename("").is_err());
    }
}
