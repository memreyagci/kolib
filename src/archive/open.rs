use std::{fs, path::Path};

use crate::{
    archive::{create::setup_db, model::Archive, utils::get_pool_by_archive_path},
    consts::DATABASE_FILE_NAME,
    error::ArchiveError,
};

pub async fn open(folder_path: impl AsRef<Path>) -> Result<Archive, ArchiveError> {
    let folder = folder_path.as_ref().to_path_buf();
    let files = get_dir_content(folder_path.as_ref())?;

    if !files.contains(&DATABASE_FILE_NAME.to_string()) {
        Err(ArchiveError::InvalidArchive { reason: (None) })
    } else {
        let pool = get_pool_by_archive_path(&folder).await?;
        let archive = Archive::new(pool, folder);

        setup_db(&archive).await?;

        Ok(archive)
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

#[cfg(test)]
mod tests {
    use crate::{
        archive::{self},
        error::ArchiveError,
        migrations::check_db_ver,
        test_helpers::{create_non_empty_dir_in_temp, init_archive_in_temp_dir},
    };

    #[tokio::test]
    async fn opening_valid_archive_succeeds() {
        let (_guard, archive_dir) = init_archive_in_temp_dir().await;
        let archive = archive::open(&archive_dir).await;

        assert!(archive.is_ok());
        assert_eq!(check_db_ver(&archive.unwrap()).await.unwrap(), 2);
    }

    #[tokio::test]
    async fn opening_invalid_archive_fails() {
        let (_guard, dir) = create_non_empty_dir_in_temp();
        let archive = archive::open(&dir).await;

        assert!(matches!(
            archive,
            Err(ArchiveError::InvalidArchive { reason: None })
        ));
    }
}
