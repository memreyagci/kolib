use std::path::Path;

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
        init_db(&archive).await?;

        Ok(archive)
    }
}

async fn init_db(archive: &Archive) -> Result<(), ArchiveError> {
    let migrations: Vec<&str> = vec![
        include_str!("../migrations/0001__initial_drizzle_schema.sql"),
        include_str!("../migrations/0002__rust_rewrite.sql"),
    ];

    for migration in migrations {
        sqlx::raw_sql(migration).execute(archive.pool()).await?;
    }

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
