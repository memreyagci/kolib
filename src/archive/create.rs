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
