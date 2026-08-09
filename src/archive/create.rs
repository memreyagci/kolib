use std::path::Path;

use crate::migrations::{Migration, check_db_ver};
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

    let mut curr_ver = check_db_ver(&archive).await?;

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

            curr_ver = check_db_ver(&archive).await?;
        }
    }

    tx.commit().await?;

    Ok(())
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
