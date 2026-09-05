use kolib::{archive::model::Archive, error::ArchiveError};

use crate::common::{create_archive_in_temp_dir, create_non_empty_dir_in_temp};

#[tokio::test]
async fn opens_existing_archive() {
    let (_guard, archive_path, archive) = create_archive_in_temp_dir().await;
    archive.close().await;

    let reopened = Archive::open(&archive_path)
        .await
        .expect("opening an existing archive should succeed");

    assert_eq!(reopened.folder(), archive_path);
}

#[tokio::test]
async fn rejects_directory_without_archive_database() {
    let (_guard, archive_path) = create_non_empty_dir_in_temp();

    let result = Archive::open(&archive_path).await;

    assert!(
        matches!(&result, Err(ArchiveError::DatabaseNotFound)),
        "unexpected result: {result:?}"
    );
}
