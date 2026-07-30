use std::path::Path;

use crate::{
    archive::{
        model::Archive,
        utils::{get_pool_by_archive_path, is_dir_empty},
    },
    consts::DATABASE_FILE_NAME,
    error::ArchiveError,
};

use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase};

/// Creates a new Koli folder with a koli.db
pub async fn create(folder_path: &Path) -> Result<Archive, ArchiveError> {
    if !is_dir_empty(&folder_path)? {
        Err(ArchiveError::DirNotEmpty)
    } else {
        init_db(&folder_path).await?;
        let pool = get_pool_by_archive_path(&folder_path).await?;

        Ok(Archive::new(pool, folder_path.to_path_buf()))
    }
}

// TODO: Add migration table, and move the .sql file in a proper dir
async fn init_db(folder_path: &Path) -> Result<(), ArchiveError> {
    let db_url_str = folder_path
        .join(DATABASE_FILE_NAME)
        .to_str()
        .ok_or(ArchiveError::DatabaseUrl)?
        .to_owned();

    match Sqlite::create_database(&db_url_str).await {
        Ok(x) => {
            let db = SqlitePool::connect(&db_url_str).await?;
            let contents = include_str!("../migrations/0000_gray_the_phantom.sql");

            sqlx::raw_sql(contents).execute(&db).await?;
            db.close().await;

            Ok(x)
        }
        Err(e) => Err(ArchiveError::SqlxError(e)),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use uuid::Uuid;

    use super::*;
    use crate::consts::DATABASE_FILE_NAME;

    // TODO: add negative tests

    // To be able to test archive folder creations in an empty dir
    fn create_an_empty_folder() -> PathBuf {
        let tmp_dir = std::env::temp_dir();
        let folder_name = Uuid::now_v7().to_string();
        let empty_dir_path = tmp_dir.join(folder_name);

        fs::create_dir(&empty_dir_path).unwrap();

        empty_dir_path
    }

    #[tokio::test]
    async fn archive_creation_in_empty_dir_works() {
        let empty_dir_path = create_an_empty_folder();
        println!("{empty_dir_path:?}");

        let result = match create(&empty_dir_path).await {
            Ok(x) => Ok(x),
            Err(e) => Err(e),
        };

        let db_path = empty_dir_path.join(DATABASE_FILE_NAME);

        assert!(result.is_ok(), "Failed because of {result:?}");

        assert!(
            fs::exists(&db_path).is_ok(),
            "File {:?} does not exist in path",
            db_path,
        );
    }
}
