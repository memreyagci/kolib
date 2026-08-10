use std::{fs, path::Path};

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::{
    archive::model::Archive,
    consts::DATABASE_FILE_NAME,
    error::ArchiveError,
    migrations::{DEPRECATED_MIGRATION_TABLA_NAME, MIGRATION_TABLE_NAME, Migration},
};

pub(crate) fn get_dir_content(folder_path: impl AsRef<Path>) -> Result<Vec<String>, ArchiveError> {
    match fs::read_dir(folder_path) {
        Err(e) => Err(ArchiveError::IoError(e)),
        Ok(paths) => {
            let mut files: Vec<String> = Vec::new();
            for path in paths {
                files.push(path?.file_name().display().to_string());
            }
            Ok(files)
        }
    }
}

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

impl Archive {
    /// Sets up the database for a given archive. It handles both initialization of an
    /// empty database, and migrations for an existing one.
    // TODO: Check if you should verify hashes here too.
    pub(super) async fn setup_db(archive: &Archive) -> Result<(), ArchiveError> {
        let mut tx = archive.pool().begin().await?;
        let mut curr_ver = archive.db_version().await?;

        let migrations = Migration::get()?;

        for m in &migrations {
            if curr_ver < m.ver() {
                sqlx::raw_sql(sqlx::AssertSqlSafe(m.file_content().clone()))
                    .execute(&mut *tx)
                    .await?;

                // Since the current migration table only exists starting with version 2, we can only insert
                // the first version's migration file details in version 2. Then, starting with version 2, we
                // can insert them by looping through.
                if curr_ver == 2 {
                    sqlx::query!(
                        "INSERT INTO kolib_migrations (version, title, checksum) VALUES (?, ?, ?)",
                        migrations[0].ver(),
                        migrations[0].title(),
                        migrations[0].hash(),
                    )
                    .execute(&mut *tx)
                    .await?;
                }
                if curr_ver >= 2 {
                    sqlx::query!(
                        "INSERT INTO kolib_migrations (version, title, checksum) VALUES (?, ?, ?)",
                        m.ver(),
                        m.title(),
                        m.hash(),
                    )
                    .execute(&mut *tx)
                    .await?;
                }

                curr_ver = archive.db_version().await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    pub(crate) async fn db_version(&self) -> Result<i64, ArchiveError> {
        let result_drizzle_migration_table = sqlx::query!(
            "SELECT name FROM sqlite_master WHERE type='table' AND name=?",
            DEPRECATED_MIGRATION_TABLA_NAME
        )
        .fetch_optional(self.pool())
        .await?;

        let result_kolib_migration_table = sqlx::query!(
            "SELECT name FROM sqlite_master WHERE type='table' AND name=?",
            MIGRATION_TABLE_NAME
        )
        .fetch_optional(self.pool())
        .await?;

        if result_drizzle_migration_table.is_some() {
            Ok(1)
        } else if result_kolib_migration_table.is_some() {
            // If the table is found, but no rows exist, it should default to 2, as this
            // table is first initialized in that version, thus its rows do not exist yet.
            let curr_ver: i64 = sqlx::query_scalar!(
                "SELECT COALESCE(MAX(version), 2) as latest_version FROM kolib_migrations"
            )
            .fetch_one(self.pool())
            .await?;

            Ok(curr_ver)
        } else {
            Ok(0)
        }
    }
}
