use std::path::Path;

use crate::migrations::Migration;
use crate::{
    archive::{
        model::Archive,
        utils::{get_pool_by_archive_path, is_dir_empty},
    },
    error::ArchiveError,
};

/// Creates a new Koli folder with a koli.db
pub async fn create(folder_path: &Path) -> Result<Archive, ArchiveError> {
    if !is_dir_empty(&folder_path)? {
        Err(ArchiveError::DirNotEmpty)
    } else {
        let pool = get_pool_by_archive_path(&folder_path).await?;
        let archive = Archive::new(pool, folder_path.to_path_buf());
        setup_db(&archive).await?;

        Ok(archive)
    }
}

/// Sets up the database for a given archive. It handles both initialization of an
/// empty database, and migrations for an existing one.
// TODO: Check if you should verify hashes here too.
pub(crate) async fn setup_db(archive: &Archive) -> Result<(), ArchiveError> {
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

#[cfg(test)]
mod tests {
    use crate::{
        archive,
        error::ArchiveError,
        test_helpers::{create_empty_dir_in_temp, create_non_empty_dir_in_temp},
    };

    #[tokio::test]
    async fn archive_creation_in_empty_dir_succeeds() {
        let (_guard, empty_dir) = create_empty_dir_in_temp();
        let archive = archive::create(&empty_dir).await;

        assert!(archive.is_ok(), "Failed because of {archive:?}");
        assert_eq!(archive.unwrap().db_version().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn archive_creation_in_non_empty_dir_fails_with_dir_not_empty() {
        let (_guard, empty_dir) = create_non_empty_dir_in_temp();
        let archive = archive::create(&empty_dir).await;

        assert!(
            matches!(archive, Err(ArchiveError::DirNotEmpty)),
            "Result was instead: {:?}",
            archive
        );
    }
}
