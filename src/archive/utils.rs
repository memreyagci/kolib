use std::{fs, path::Path};

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::{
    archive::model::Archive,
    consts::DATABASE_FILE_NAME,
    error::ArchiveError,
    migrations::{DEPRECATED_MIGRATION_TABLA_NAME, MIGRATION_TABLE_NAME, Migration},
};

pub(crate) fn is_dir_empty(folder_path: impl AsRef<Path>) -> Result<bool, ArchiveError> {
    Ok(fs::read_dir(folder_path)?.next().is_none())
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
    // TODO: tw_dm subtable rows need to be generated out of the first version's single row
    // arrays.
    pub(super) async fn setup_db(archive: &Archive) -> Result<(), ArchiveError> {
        let migrations = Migration::get()?;
        let mut curr_ver = archive.db_version().await?;

        let mut tx = archive.pool().begin().await?;

        for m in &migrations {
            if curr_ver >= m.ver() {
                continue;
            }

            sqlx::raw_sql(sqlx::AssertSqlSafe(m.file_content().clone()))
                .execute(&mut *tx)
                .await?;

            // Since the current migration table only exists starting with version 2, we can only insert
            // the first version's migration file details in version 2. Then, starting with version 2, we
            // can insert them by looping through.
            if m.ver() == 2 {
                let first_migration = &migrations[0];

                sqlx::query!(
                    "INSERT INTO kolib_migrations (version, title, checksum) VALUES (?, ?, ?)",
                    first_migration.ver(),
                    first_migration.title(),
                    first_migration.hash(),
                )
                .execute(&mut *tx)
                .await?;
            }
            if m.ver() >= 2 {
                sqlx::query!(
                    "INSERT INTO kolib_migrations (version, title, checksum) VALUES (?, ?, ?)",
                    m.ver(),
                    m.title(),
                    m.hash(),
                )
                .execute(&mut *tx)
                .await?;
            }

            curr_ver = m.ver();
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

        if result_drizzle_migration_table.is_some() {
            return Ok(1);
        }

        let result_kolib_migration_table = sqlx::query!(
            "SELECT name FROM sqlite_master WHERE type='table' AND name=?",
            MIGRATION_TABLE_NAME
        )
        .fetch_optional(self.pool())
        .await?;

        if result_kolib_migration_table.is_none() {
            return Ok(0);
        }

        let curr_ver: i64 = sqlx::query_scalar!(
            "SELECT COALESCE(MAX(version), 2) as latest_version FROM kolib_migrations"
        )
        .fetch_one(self.pool())
        .await?;

        Ok(curr_ver)
    }
}
