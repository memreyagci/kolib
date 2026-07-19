use super::ManifestFile;
use crate::error::ArchiveError;

use chrono::Utc;
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase};
use std::fs;
use uuid::Uuid;

const MANIFEST_FILE_NAME: &str = "koli.json";
const DATABASE_FILE_NAME: &str = "koli.db";
const MANIFEST_FORMAT_VER: u8 = 1;

/// Creates a new Koli folder, which has:
/// - koli.json (likely to be deprecated with a db table later on)
/// - koli.db
pub async fn create(folder_path: &str) -> Result<(), ArchiveError> {
    if !is_dir_empty(&folder_path)? {
        Err(ArchiveError::DirNotEmpty)
    } else {
        create_manifest_file(&folder_path)?;
        init_db(&folder_path).await?;
        //TODO: 3. Done. Consider returning the path or something
        Ok(())
    }
}

fn is_dir_empty(folder_path: &str) -> Result<bool, ArchiveError> {
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

fn create_manifest_file(folder_path: &str) -> Result<(), ArchiveError> {
    let manifest_content = ManifestFile {
        r#type: String::from("koli"),
        formatVersion: MANIFEST_FORMAT_VER,
        id: Uuid::now_v7(),
        createdAt: Utc::now(),
    };

    match serde_json::to_string(&manifest_content) {
        Err(e) => Err(ArchiveError::SerdeError(e)),
        Ok(x) => {
            fs::write(format!("{folder_path}{}", MANIFEST_FILE_NAME), &x).unwrap();
            Ok(())
        }
    }
}

// TODO: Add migration table, and move the sql file in a proper dir
async fn init_db(folder_path: &str) -> Result<(), ArchiveError> {
    let db_url = format!("sqlite://{folder_path}{DATABASE_FILE_NAME}");

    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        match Sqlite::create_database(&db_url).await {
            Ok(x) => {
                let db = SqlitePool::connect(&db_url).await.unwrap();
                let contents = include_str!("../0000_gray_the_phantom.sql");

                sqlx::raw_sql(contents).execute(&db).await?;
                db.close().await;

                Ok(x)
            }
            Err(e) => Err(ArchiveError::SqlxError(e)),
        }
    } else {
        Err(ArchiveError::KoliDbAlreadyExists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add negative tests

    // To be able to test archive folder creations in an empty dir
    fn create_an_empty_folder() -> String {
        let tmp_dir = std::env::temp_dir().display().to_string();
        let folder_name = Uuid::now_v7().to_string();
        let empty_dir_path = format!("{tmp_dir}{folder_name}/");

        fs::create_dir(&empty_dir_path).unwrap();

        empty_dir_path
    }

    #[tokio::test]
    async fn archive_creation_in_empty_dir_works() {
        let empty_dir_path = create_an_empty_folder();
        println!("{empty_dir_path}");

        let result = match create(&empty_dir_path).await {
            Ok(x) => Ok(x),
            Err(e) => Err(e),
        };

        let files_to_check = vec![
            String::from(format!("{empty_dir_path}koli.db")),
            String::from(format!("{empty_dir_path}koli.json")),
        ];

        assert!(result.is_ok(), "Failed because of {result:?}");

        for file in files_to_check {
            assert!(
                fs::exists(file.to_string()).unwrap(),
                "File {} does not exist in path",
                file.to_string(),
            );
        }
    }
}
