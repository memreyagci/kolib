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

#[tokio::test]
async fn rejects_archive_from_newer_kolib_version() {
    let (_guard, archive_path, archive) = create_archive_in_temp_dir().await;

    sqlx::query(
        r#"
        INSERT INTO kolib_migrations (version, title, checksum)
        VALUES (999, 'future_migration', 'future_checksum')
        "#,
    )
    .execute(archive.pool())
    .await
    .expect("adding a future migration record should succeed");

    archive.close().await;

    let result = Archive::open(&archive_path).await;

    assert!(
        matches!(
            &result,
            Err(ArchiveError::UnsupportedVersion {
                archive_version: 999,
                latest_supported_version: 2,
            })
        ),
        "unexpected result: {result:?}"
    );
}
