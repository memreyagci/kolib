use std::{io, num::ParseIntError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("I/O error occurred")]
    IoError(#[from] io::Error),

    #[error("Parse int error")]
    ParseIntError(#[from] ParseIntError),

    #[error("Directory is not empty")]
    DirNotEmpty,

    #[error("sqlx related error occurred")]
    SqlxError(#[from] sqlx::Error),

    #[error("Database already exists")]
    KoliDbAlreadyExists,

    #[error("koli.db is not found")]
    InvalidArchive { reason: Option<String> },

    #[error("Database URL could not be created")]
    DatabaseUrl,

    #[error("Migration error occured")]
    MigrationError(#[from] MigrationError),
}

#[derive(Error, Debug)]
pub enum ExportReaderError {
    #[error("{export_file_path} is not found.")]
    ExportFileNotFound { export_file_path: String },

    #[error("invalid enum value")]
    StrumError(#[from] strum::ParseError),

    #[error("I/O error occurred")]
    IoError(#[from] io::Error),

    #[error("Invalid or unsupport file for {platform}: {file_name}")]
    InvalidOrUnsupportedFileName { platform: String, file_name: String },

    #[error("file must be set.")]
    FileNotFound,

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

    #[error("sqlx related error occurred")]
    SqlxError(#[from] sqlx::Error),

    #[error("regex related error occurred")]
    RegexError(#[from] regex::Error),

    #[error("failed to deserialize export: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("chrono related error occurred")]
    ChronoError(#[from] chrono::ParseError),

    #[error("media path could not be parsed")]
    MediaPathParseError,

    #[error("url related error occurred")]
    UrlError(#[from] url::ParseError),
}

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Account name cannot be empty or contain only whitespace.")]
    InvalidName,

    #[error("sqlx error occured.")]
    SqlxError(#[from] sqlx::Error),

    #[error("uuid error occured.")]
    UuidError(#[from] uuid::Error),

    #[error("strum error occured.")]
    StrumError(#[from] strum::ParseError),

    #[error("I/O error occurred")]
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

    #[error(
        "kolib checks for __drizzle_migrations and kolib_migrations table to determine the version, and neither of them found."
    )]
    MigrationTableNotFound,
}
