use std::path::Path;

use crate::{
    archive::{
        model::Archive,
        utils::{get_pool_by_archive_path, is_dir_empty},
    },
    error::ArchiveError,
};

impl Archive {
    /// Creates a new Koli archive in an empty folder with a koli.db
    pub async fn create(folder_path: &Path) -> Result<Archive, ArchiveError> {
        if !is_dir_empty(folder_path)? {
            return Err(ArchiveError::DirNotEmpty);
        }

        let pool = get_pool_by_archive_path(folder_path).await?;
        let archive = Self::new(pool, folder_path.to_path_buf());

        Self::setup_db(&archive).await?;

        Ok(archive)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        archive::model::Archive,
        error::ArchiveError,
        test_helpers::{create_empty_dir_in_temp, create_non_empty_dir_in_temp},
    };

    #[tokio::test]
    async fn archive_creation_in_empty_dir_succeeds() {
        let (_guard, empty_dir) = create_empty_dir_in_temp();
        let archive = Archive::create(&empty_dir).await;

        assert!(archive.is_ok(), "Failed because of {archive:?}");
        assert_eq!(archive.unwrap().db_version().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn archive_creation_in_non_empty_dir_fails_with_dir_not_empty() {
        let (_guard, empty_dir) = create_non_empty_dir_in_temp();
        let archive = Archive::create(&empty_dir).await;

        assert!(
            matches!(archive, Err(ArchiveError::DirNotEmpty)),
            "Result was instead: {:?}",
            archive
        );
    }
}
