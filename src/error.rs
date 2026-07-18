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
}
