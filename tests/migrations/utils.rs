use std::{
    fs, io,
    path::{Path, PathBuf},
};

use kolib::archive::model::Archive;
use tempfile::TempDir;

use crate::common::{create_empty_dir_in_temp, fixture_path};

pub(super) fn copy_fixture_to_temp(relative_path: impl AsRef<Path>) -> (TempDir, PathBuf) {
    fn copy_contents(source: &Path, destination: &Path) -> io::Result<()> {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let destination_path = destination.join(entry.file_name());

            if entry.file_type()?.is_dir() {
                fs::create_dir(&destination_path)?;
                copy_contents(&entry.path(), &destination_path)?;
            } else {
                fs::copy(entry.path(), destination_path)?;
            }
        }

        Ok(())
    }

    let fixture = fixture_path(relative_path);
    let (guard, archive_path) = create_empty_dir_in_temp();

    copy_contents(&fixture, &archive_path).expect("copying fixture should succeed");

    (guard, archive_path)
}

pub(super) async fn migration_versions(archive: &Archive) -> Vec<i64> {
    sqlx::query_scalar::<_, i64>("SELECT version FROM kolib_migrations ORDER BY version")
        .fetch_all(archive.pool())
        .await
        .expect("reading migration versions should succeed")
}

pub(super) async fn twitter_dm_row_counts(archive: &Archive) -> (i64, i64, i64, i64) {
    sqlx::query_as::<_, (i64, i64, i64, i64)>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM twitter_direct_messages),
            (SELECT COUNT(*) FROM twitter_dm_reactions),
            (SELECT COUNT(*) FROM twitter_dm_edit_history),
            (SELECT COUNT(*) FROM twitter_dm_attachments)
        "#,
    )
    .fetch_one(archive.pool())
    .await
    .expect("counting migrated rows should succeed")
}
