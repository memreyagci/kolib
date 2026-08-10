#![cfg(test)]

use std::{fs::File, path::PathBuf};

use tempfile::TempDir;

use crate::archive::model::Archive;

pub fn create_empty_dir_in_temp() -> (TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_dir_path = temp_dir.path().to_path_buf();

    (temp_dir, temp_dir_path)
}

pub fn create_non_empty_dir_in_temp() -> (TempDir, PathBuf) {
    let (temp_dir, temp_dir_path) = create_empty_dir_in_temp();
    File::create(temp_dir_path.join("whatever.txt")).unwrap();

    (temp_dir, temp_dir_path)
}

pub async fn init_archive_in_temp_dir() -> (TempDir, PathBuf, Archive) {
    let (temp_dir, temp_dir_path) = create_empty_dir_in_temp();
    let archive = Archive::create(&temp_dir_path).await.unwrap();

    (temp_dir, temp_dir_path, archive)
}
