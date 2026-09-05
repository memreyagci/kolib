use std::{io, num::ParseIntError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Directory is not empty")]
    DirNotEmpty,

    #[error("database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("koli.db is not found")]
    InvalidArchive { reason: Option<String> },

    #[error(transparent)]
    MigrationError(#[from] MigrationError),
}

#[derive(Error, Debug)]
pub enum ExportReaderError {
    #[error("{export_file_path} is not found.")]
    ExportFileNotFound { export_file_path: String },

    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("{imported_filename} file is not supported by {importer_name}")]
    InvalidFilename {
        imported_filename: String,
        importer_name: String,
    },

    #[error("Account and importer platform doesn't match: {acc_platform} & {importer_platform}")]
    PlatformMismatch {
        acc_platform: String,
        importer_platform: String,
    },

    #[error("database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("failed to deserialize export: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("media path could not be parsed")]
    MediaPathParseError,

    #[error("url error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error(transparent)]
    Twitter(#[from] TwitterError),
}

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Account name cannot be empty or contain only whitespace.")]
    InvalidName,

    #[error("database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("uuid error occured.")]
    UuidError(#[from] uuid::Error),

    #[error("strum error occured.")]
    StrumError(#[from] strum::ParseError),

    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("Parse int error")]
    ParseIntError(#[from] ParseIntError),

    #[error("Migration version could not be derived from {filename}")]
    DeriveMigrationVersionError { filename: String },

    #[error("Migration title could not be derived from {filename}")]
    DeriveMigrationTitleError { filename: String },

    #[error("Expected hash: {expected_hash:?}, actual hash of file: {actual_hash}")]
    MigrationFileHashMismatch {
        expected_hash: String,
        actual_hash: String,
    },
}

#[derive(Debug, Error)]
pub enum TwitterError {
    #[error("conversation `{conversation_id}` was not found for account `{account_id}`")]
    ConversationNotFound {
        account_id: String,
        conversation_id: String,
    },
}
