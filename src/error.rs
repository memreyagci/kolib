use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("I/O error occurred")]
    IoError(#[from] io::Error),

    #[error("Directory is not empty")]
    DirNotEmpty,

    #[error("serde_json related error occurred")]
    SerdeError(#[from] serde_json::Error),

    #[error("sqlx related error occurred")]
    SqlxError(#[from] sqlx::Error),

    #[error("Database already exists")]
    KoliDbAlreadyExists,

    #[error("koli.db is not found")]
    InvalidArchive { reason: Option<String> },

    #[error("Database URL could not be created")]
    DatabaseUrl,

    #[error(
        "kolib checks for __drizzle_migrations and kolib_migrations table to determine the version, and neither of them found."
    )]
    MigrationTableNotFound,
}

#[derive(Error, Debug)]
pub enum ExportReaderError {
    #[error("I/O error occurred")]
    IoError(#[from] io::Error),

    #[error("Invalid or unsupport file for {platform}: {file_name}")]
    InvalidOrUnsupportedFileName { platform: String, file_name: String },

    #[error("file must be set.")]
    FileNotFound,

    #[error("Account and importer platform doesn't match: {acc_platform} & {importer_platform}")]
    PlatformMismatch {
        acc_platform: String,
        importer_platform: String,
    },

    #[error("sqlx related error occurred")]
    SqlxError(#[from] sqlx::Error),
}

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Account name is not set.")]
    AccountNameNull,

    #[error("sqlx error occured.")]
    SqlxError(#[from] sqlx::Error),

    #[error("uuid error occured.")]
    UuidError(#[from] uuid::Error),

    #[error("strum error occured.")]
    StrumError(#[from] strum::ParseError),

    #[error("I/O error occurred")]
    IoError(#[from] io::Error),
}
