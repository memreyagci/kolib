use std::{fs, path::Path};

use crate::{
    archive::{model::Archive, utils::get_pool_by_archive_path},
    consts::DATABASE_FILE_NAME,
    error::ArchiveError,
};

pub async fn open(folder_path: impl AsRef<Path>) -> Result<Archive, ArchiveError> {
    let folder = folder_path.as_ref().to_path_buf();
    let files = get_dir_content(folder_path.as_ref())?;

    if !files.contains(&DATABASE_FILE_NAME.to_string()) {
        Err(ArchiveError::InvalidArchive { reason: (None) })
    } else {
        // TODO: Check db ver and run migrations
        let pool = get_pool_by_archive_path(&folder).await?;
        Ok(Archive::new(pool, folder))
    }
}

fn get_dir_content(folder_path: impl AsRef<Path>) -> Result<Vec<String>, ArchiveError> {
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
