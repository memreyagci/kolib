use std::path::Path;

use crate::{
    archive::{model::Archive, utils::get_pool_by_archive_path},
    consts::DATABASE_FILE_NAME,
    error::ArchiveError,
};

impl Archive {
    pub async fn open(folder_path: impl AsRef<Path>) -> Result<Self, ArchiveError> {
        let folder = folder_path.as_ref().to_path_buf();

        let db_file_path = folder.join(DATABASE_FILE_NAME);
        if !db_file_path.is_file() {
            return Err(ArchiveError::DatabaseNotFound);
        }

        let pool = get_pool_by_archive_path(&folder).await?;
        let archive = Self::new(pool, folder);

        Self::setup_db(&archive).await?;

        Ok(archive)
    }
}
