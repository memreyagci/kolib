use crate::error::ArchiveError;
use crate::{archive::model::Archive, error::MigrationError};

use hex_literal::hex;
use sha2::{Digest, Sha256};

const DEPRECATED_MIGRATION_TABLA_NAME: &str = "__drizzle_migrations";
const MIGRATION_TABLE_NAME: &str = "kolib_migrations";

const MIGRATIONS: &[(&str, &str, [u8; 32])] = &[
    (
        "0001__initial_drizzle_schema.sql",
        include_str!("./migrations/0001__initial_drizzle_schema.sql"),
        hex!("6501bf8ee7822f90ec32da6c2c63393396b07dcff0119c9442f179e08b3ded8b"),
    ),
    (
        "0002__rust_rewrite.sql",
        include_str!("./migrations/0002__rust_rewrite.sql"),
        hex!("6ff2c0b1f465ab222ef70097dd10034315ddd26dc0773946b9cde3bd050c94cb"),
    ),
];

#[derive(Debug)]
pub struct Migration {
    title: String,
    ver: i64,
    file_content: String,
    hash: String,
}

impl Migration {
    /// Returns a vector of Migration instances, which have essential information to be
    /// used in database migrations. It makes sure the hash of the file matches the
    /// one in constants, and instances are ordered by "ver".
    pub(crate) fn get() -> Result<Vec<Migration>, MigrationError> {
        let mut migrations = Vec::new();

        for (filename, content, hash) in MIGRATIONS {
            let title: String = filename
                .split("__")
                .nth(1)
                .ok_or(MigrationError::DeriveMigrationTitleError {
                    filename: filename.to_string(),
                })?
                .strip_suffix(".sql")
                .ok_or(MigrationError::DeriveMigrationTitleError {
                    filename: filename.to_string(),
                })?
                .to_string();

            let ver: i64 = filename
                .split("__")
                .next()
                .ok_or(MigrationError::DeriveMigrationVersionError {
                    filename: filename.to_string(),
                })?
                .parse::<i64>()?;

            let calculated_hash = Sha256::digest(content);

            if calculated_hash.as_slice() != hash {
                return Err(MigrationError::MigrationFileHashMismatch {
                    expected_hash: hex::encode(calculated_hash),
                    actual_hash: hex::encode(hash),
                });
            } else {
                migrations.push(Migration {
                    title,
                    ver,
                    file_content: content.to_string(),
                    hash: hex::encode(calculated_hash),
                });
            }
        }
        migrations.sort_by_key(|m| m.ver);

        Ok(migrations)
    }

    pub(crate) fn title(&self) -> &String {
        &self.title
    }
    pub(crate) fn ver(&self) -> i64 {
        self.ver
    }
    pub(crate) fn file_content(&self) -> &String {
        &self.file_content
    }
    pub(crate) fn hash(&self) -> &String {
        &self.hash
    }
}

pub(crate) async fn check_db_ver(archive: &Archive) -> Result<i64, ArchiveError> {
    let result_drizzle_migration_table = sqlx::query!(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?",
        DEPRECATED_MIGRATION_TABLA_NAME
    )
    .fetch_optional(archive.pool())
    .await?;

    let result_kolib_migration_table = sqlx::query!(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?",
        MIGRATION_TABLE_NAME
    )
    .fetch_optional(archive.pool())
    .await?;

    if result_drizzle_migration_table.is_some() {
        Ok(1)
    } else if result_kolib_migration_table.is_some() {
        // If the table is found, but no rows exist, it should default to 2, as this
        // table is first initialized in that version, thus its rows do not exist yet.
        let curr_ver: i64 = sqlx::query_scalar!(
            "SELECT COALESCE(MAX(version), 2) as latest_version FROM kolib_migrations"
        )
        .fetch_one(archive.pool())
        .await?;

        Ok(curr_ver)
    } else {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use crate::archive;

    use super::*;

    #[tokio::test]
    async fn check_db_ver_returns_correct_ver() {
        fs::create_dir("/private/var/tmp/test_1/").unwrap();
        let dir = Path::new("/private/var/tmp/test_1/");

        let arc = archive::create(&dir).await.unwrap();
        let version = check_db_ver(arc).await.unwrap();

        println!("Result is: {version:?}");

        assert_eq!(version, 2);
    }
}
