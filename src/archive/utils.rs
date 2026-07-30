use std::{fs, path::Path};

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::{consts::DATABASE_FILE_NAME, error::ArchiveError};

pub(crate) fn is_dir_empty(folder_path: impl AsRef<Path>) -> Result<bool, ArchiveError> {
    match fs::read_dir(folder_path) {
        Err(e) => Err(ArchiveError::IoError(e)),
        Ok(paths) => {
            if paths.count() == 0 {
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}

pub(crate) async fn get_pool_by_archive_path(folder: &Path) -> Result<SqlitePool, ArchiveError> {
    let db_file_path = folder.join(DATABASE_FILE_NAME);

    let conn_opts = SqliteConnectOptions::new()
        .filename(&db_file_path)
        .create_if_missing(true)
        .busy_timeout(std::time::Duration::from_millis(5000));

    Ok(SqlitePool::connect_with(conn_opts).await?)
}
