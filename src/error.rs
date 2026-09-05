use std::{io, num::ParseIntError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Directory is not empty")]
    DirNotEmpty,

    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("koli.db is not found")]
    InvalidArchive { reason: Option<String> },

    #[error(transparent)]
    Migration(#[from] MigrationError),
}

#[derive(Error, Debug)]
pub enum ExportReaderError {
    #[error("{export_file_path} is not found.")]
    InvalidExportPath { export_file_path: String },

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("{expected} file is not supported by {actual}")]
    UnexpectedFilename { expected: String, actual: String },

    #[error("Account and importer platform doesn't match: {acc_platform} & {importer_platform}")]
    PlatformMismatch {
        acc_platform: String,
        importer_platform: String,
    },

    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("failed to deserialize export: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("media path could not be parsed")]
    MediaPathParse,

    #[error("url error: {0}")]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Twitter(#[from] TwitterError),
}

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("account `{account_id}` was not found")]
    NotFound { account_id: String },

    #[error("Account name cannot be empty or contain only whitespace.")]
    InvalidName,

    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("uuid error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("strum error: {0}")]
    Strum(#[from] strum::ParseError),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("parse int error: {0}")]
    ParseInt(#[from] ParseIntError),

    #[error("Migration version could not be derived from {filename}")]
    DeriveMigrationVersion { filename: String },

    #[error("Migration title could not be derived from {filename}")]
    DeriveMigrationTitle { filename: String },

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
