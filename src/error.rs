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

    #[error("archive database `koli.db` was not found")]
    DatabaseNotFound,

    #[error(
        "archive version `{archive_version}` is newer than the latest supported version `{latest_supported_version}`: update kolib to open it"
    )]
    UnsupportedVersion {
        archive_version: i64,
        latest_supported_version: i64,
    },

    #[error(transparent)]
    Migration(#[from] MigrationError),
}

#[derive(Error, Debug)]
pub enum ExportReaderError {
    #[error("Invalid export path: {export_file_path}")]
    InvalidExportPath { export_file_path: String },

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("expected export filename `{expected}`, got `{actual}`")]
    UnexpectedFilename { expected: String, actual: String },

    #[error("Account and importer platform doesn't match: {acc_platform} & {importer_platform}")]
    PlatformMismatch {
        acc_platform: String,
        importer_platform: String,
    },

    #[error("platform `{platform}` is not supported for imports")]
    UnsupportedPlatform { platform: String },

    #[error(
        "dataset `{dataset_type}` already exists for account `{account_id}`: merging and replacing datasets are not yet supported"
    )]
    DatasetAlreadyExists {
        account_id: String,
        dataset_type: String,
    },

    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("date/time parse error: {0}")]
    DateTime(#[from] chrono::ParseError),

    #[error("integer parse error: {0}")]
    ParseInt(#[from] ParseIntError),

    #[error("failed to deserialize export: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("media path could not be parsed")]
    MediaPathParse,

    #[error("media filename `{filename}` is invalid")]
    InvalidMediaFilename { filename: String },

    #[error("url error: {0}")]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Message(#[from] MessageError),

    #[error(transparent)]
    Contact(#[from] ContactError),

    #[error("strum error: {0}")]
    Strum(#[from] strum::ParseError),
}

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("account `{account_id}` was not found")]
    NotFound { account_id: String },

    #[error("Account name cannot be empty or contain only whitespace.")]
    InvalidName,

    #[error("This file is not supported: {filename}")]
    InvalidDatasetType { filename: String },

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
pub enum ContactError {
    #[error("contact `{contact_id}` was not found")]
    NotFound { contact_id: String },

    #[error("the built-in Me contact was not found")]
    MeNotFound,

    #[error("contact name cannot be empty or contain only whitespace")]
    InvalidName,

    #[error("the built-in Me contact cannot be renamed")]
    CannotRenameMe,

    #[error("the built-in Me contact cannot be deleted")]
    CannotDeleteMe,

    #[error(
        "participant `{participant_key}` is not a sender in conversation `{conversation_id}` for account `{account_id}`"
    )]
    MessageParticipantNotFound {
        account_id: String,
        conversation_id: String,
        participant_key: String,
    },

    #[error("message sender contact data is incomplete")]
    IncompleteMessageContact,

    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("uuid error: {0}")]
    Uuid(#[from] uuid::Error),
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
pub enum MessageError {
    #[error("conversation `{conversation_id}` was not found for account `{account_id}`")]
    ConversationNotFound {
        account_id: String,
        conversation_id: String,
    },

    #[error("conversation display name cannot be empty or contain only whitespace")]
    InvalidConversationDisplayName,

    #[error("message `{message_id}` was not found for account `{account_id}`")]
    NotFound {
        account_id: String,
        message_id: String,
    },

    #[error("message `{message_id}` is beyond the supported pagination range")]
    PageIndexOutOfRange { message_id: String },
}

#[derive(Debug, Error)]
pub enum TimestampError {
    #[error("Unix timestamp `{milliseconds}` milliseconds is outside Chrono's supported range")]
    OutOfRange { milliseconds: i64 },

    #[error("invalid timestamp format: {0}")]
    InvalidFormat(#[from] chrono::format::ParseError),
}
