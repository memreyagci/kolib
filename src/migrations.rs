use crate::consts::DATABASE_FILE_NAME;
use sqlx::{Row, Sqlite, SqlitePool, migrate::MigrateDatabase};

use crate::error::ArchiveError;

const DEPRECATED_MIGRATION_TABLA_NAME: &str = "__drizzle_migrations";
const MIGRATION_TABLE_NAME: &str = "koli_migrations";

/// Checks the current database version.
/// The implementation is incomplete. Right now, it checks whether drizzle migration table,
/// which is from the old TypeScript implementation of this project, exists. If so,
/// the version is 1. Otherwise, it returns 2, although it is supposed check the version
/// from the new table, which is not ready yet.
pub async fn check_db_ver(folder_path: &str) -> Result<u8, ArchiveError> {
    let db_url = format!("sqlite://{folder_path}{DATABASE_FILE_NAME}");

    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Err(ArchiveError::InvalidArchive {
            reason: Some(format!("{DATABASE_FILE_NAME} is not found")),
        })
    } else {
        let db = SqlitePool::connect(&db_url).await.unwrap();

        let result_drizzle_migration_table =
            sqlx::query::<_>("SELECT name FROM sqlite_master WHERE type='table' AND name=?")
                .bind(DEPRECATED_MIGRATION_TABLA_NAME)
                .fetch_one(&db)
                .await?;

        if result_drizzle_migration_table.is_empty() {
            let result_migration_table =
                sqlx::query::<_>("SELECT name FROM sqlite_master WHERE type='table' AND name=?")
                    .bind(MIGRATION_TABLE_NAME)
                    .fetch_one(&db)
                    .await?;
            if result_migration_table.is_empty() {
                Err(ArchiveError::InvalidArchive {
                    reason: Some(String::from("Migration table could not be found")),
                })
            } else {
                // TODO: Check version from the new table and return that rather than hardcoded 2
                Ok(2)
            }
        } else {
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        let version = check_db_ver("/Users/emre/Documents/repos/kolib/")
            .await
            .unwrap();

        println!("Result is: {version:?}");

        assert_eq!(version, 1);
    }
}
