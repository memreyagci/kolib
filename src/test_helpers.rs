#![cfg(test)]

use std::{fs::File, path::PathBuf};

use tempfile::TempDir;

pub fn create_empty_dir_in_tmp() -> (TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_dir_path = temp_dir.path().to_path_buf();

    (temp_dir, temp_dir_path)
}

pub fn create_non_empty_dir_in_tmp() -> (TempDir, PathBuf) {
    let (temp_dir, temp_dir_path) = create_empty_dir_in_tmp();
    File::create(temp_dir_path.join("whatever.txt")).unwrap();

    (temp_dir, temp_dir_path)
}
