use std::path::Path;

use crate::{
    archive::{
        model::Archive,
        utils::{get_dir_content, get_pool_by_archive_path},
    },
    consts::DATABASE_FILE_NAME,
    error::ArchiveError,
};

impl Archive {
    pub async fn open(folder_path: impl AsRef<Path>) -> Result<Self, ArchiveError> {
        let folder = folder_path.as_ref().to_path_buf();
        let files = get_dir_content(folder_path.as_ref())?;

        if !files.iter().any(|file| file == DATABASE_FILE_NAME) {
            return Err(ArchiveError::InvalidArchive { reason: (None) });
        }

        let pool = get_pool_by_archive_path(&folder).await?;
        let archive = Self::new(pool, folder);

        Self::setup_db(&archive).await?;

        Ok(archive)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        archive::model::Archive,
        error::ArchiveError,
        test_helpers::{create_non_empty_dir_in_temp, init_archive_in_temp_dir},
    };

    #[tokio::test]
    async fn opening_valid_archive_succeeds() {
        let (_guard, archive_dir, _) = init_archive_in_temp_dir().await;
        let archive = Archive::open(&archive_dir).await;

        assert!(archive.is_ok());
        assert_eq!(archive.unwrap().db_version().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn opening_invalid_archive_fails() {
        let (_guard, dir) = create_non_empty_dir_in_temp();
        let archive = Archive::open(&dir).await;

        assert!(matches!(
            archive,
            Err(ArchiveError::InvalidArchive { reason: None })
        ));
    }
}
