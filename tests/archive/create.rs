use kolib::{archive::model::Archive, error::ArchiveError};

use crate::common::{create_empty_dir_in_temp, create_non_empty_dir_in_temp};

#[tokio::test]
async fn creates_archive_in_empty_directory() {
    let (_guard, archive_path) = create_empty_dir_in_temp();

    let archive = Archive::create(&archive_path)
        .await
        .expect("creating an archive in an empty directory should succeed");

    assert_eq!(archive.folder(), archive_path);
    assert!(archive_path.join("koli.db").is_file());
}

#[tokio::test]
async fn rejects_archive_creation_in_non_empty_directory() {
    let (_guard, archive_path) = create_non_empty_dir_in_temp();

    let result = Archive::create(&archive_path).await;

    assert!(
        matches!(result, Err(ArchiveError::DirNotEmpty)),
        "unexpected result: {result:?}"
    );
}
