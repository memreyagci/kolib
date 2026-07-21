use std::fs;

use crate::{consts::DATABASE_FILE_NAME, error::ArchiveError, migrations::check_db_ver};

pub async fn open(folder_path: &str) -> Result<(), ArchiveError> {
    let files: Vec<String> = get_dir_content(&folder_path)?;

    if !files.contains(&DATABASE_FILE_NAME.to_string()) {
        Err(ArchiveError::InvalidArchive { reason: (None) })
    } else {
        let _curr_db_ver = check_db_ver(&folder_path).await.unwrap();
        Ok(())
    }
}

fn get_dir_content(folder_path: &str) -> Result<Vec<String>, ArchiveError> {
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
