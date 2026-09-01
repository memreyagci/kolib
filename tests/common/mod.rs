use std::{
    fs::File,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

use kolib::{archive::model::Archive, export_reader::account::models::Account, types::Platform};

pub fn fixture_path(relative_path: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(relative_path)
}

pub fn twitter_dm_fixture(scenario: &str) -> PathBuf {
    fixture_path(
        PathBuf::from("twitter")
            .join("direct_messages")
            .join(scenario)
            .join("direct-messages.js"),
    )
}

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

pub async fn create_archive_in_temp_dir() -> (TempDir, PathBuf, Archive) {
    let (temp_dir, temp_dir_path) = create_empty_dir_in_temp();
    let archive = Archive::create(&temp_dir_path).await.unwrap();

    (temp_dir, temp_dir_path, archive)
}

pub async fn create_account_in_temp_dir(
    platform: Platform,
) -> (TempDir, PathBuf, Archive, Account) {
    let (_guard, archive_path, archive) = create_archive_in_temp_dir().await;

    let account = Account::create(&archive, "test", platform).await.unwrap();

    (_guard, archive_path, archive, account)
}
