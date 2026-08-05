use crate::archive::model::Archive;
use crate::error::ArchiveError;

const DEPRECATED_MIGRATION_TABLA_NAME: &str = "__drizzle_migrations";
const MIGRATION_TABLE_NAME: &str = "kolib_migrations";

async fn check_db_ver(archive: Archive) -> Result<i64, ArchiveError> {
    let result_drizzle_migration_table = sqlx::query!(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?",
        DEPRECATED_MIGRATION_TABLA_NAME
    )
    .fetch_optional(archive.pool())
    .await?;

    let result_kolib_migration_table = sqlx::query!(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?",
        MIGRATION_TABLE_NAME
    )
    .fetch_optional(archive.pool())
    .await?;

    if result_drizzle_migration_table.is_some() {
        Ok(1)
    } else if result_kolib_migration_table.is_some() {
        let curr_ver: i64 = sqlx::query_scalar!(
            "SELECT COALESCE(MAX(version), 0) as latest_version FROM kolib_migrations"
        )
        .fetch_one(archive.pool())
        .await?;

        Ok(curr_ver)
    } else {
        Err(ArchiveError::MigrationTableNotFound)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use crate::archive;

    use super::*;

    #[tokio::test]
    async fn check_db_ver_returns_correct_ver() {
        fs::create_dir("/private/var/tmp/test_1/").unwrap();
        let dir = Path::new("/private/var/tmp/test_1/");

        let arc = archive::create(&dir).await.unwrap();
        let version = check_db_ver(arc).await.unwrap();

        println!("Result is: {version:?}");

        assert_eq!(version, 2);
    }
}
