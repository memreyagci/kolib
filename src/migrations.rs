use crate::error::MigrationError;

use hex_literal::hex;
use sha2::{Digest, Sha256};

pub(crate) const DEPRECATED_MIGRATION_TABLA_NAME: &str = "__drizzle_migrations";
pub(crate) const MIGRATION_TABLE_NAME: &str = "kolib_migrations";

const MIGRATIONS: &[(&str, &str, [u8; 32])] = &[
    (
        "0001__initial_drizzle_schema.sql",
        include_str!("./migrations/0001__initial_drizzle_schema.sql"),
        hex!("6501bf8ee7822f90ec32da6c2c63393396b07dcff0119c9442f179e08b3ded8b"),
    ),
    (
        "0002__rust_rewrite.sql",
        include_str!("./migrations/0002__rust_rewrite.sql"),
        hex!("a30926cb1846a1356bd6fff54b8829397de742d0b40431d09a4cb5db337973d8"),
    ),
];

// TODO: Consider, at some point, removing migrations, and rebuilding off of raw
// files. Since deletions, favorites, other_user_id, etc., will be in separate
// tables, they would be able to be preserved. Only problem is custom id (UUIDs)
// within datasets. Reconsider using account.id + dataset-specific identifier (which
// can be one or more values provided by the platform export file). It will be
// useful for inserting export file on top an existing dataset too, since UUID
// won't be the identifier, but the platform-provided one(s) will be anyways.
// TODO: Store raw files in db as blobs.

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
            }

            migrations.push(Migration {
                title,
                ver,
                file_content: content.to_string(),
                hash: hex::encode(calculated_hash),
            });
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
