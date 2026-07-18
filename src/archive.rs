//! This file deals with Koli archive folder related operations.
//!
//! A Koli \[archive\] folder is a folder user chooses to save their data. Everything, from
//! the sqlite database to raw imports and media files are stored in this folder.
//! Users can create a new Koli folder, or choose an existing one.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

type Uuidv7 = Uuid;
type IsoDateTime = DateTime<Utc>;

// TODO: Consider having these in an env file.
struct KoliFolderFiles {
    manifest_file_name: String,
    database_file_name: String,
    account_dir_name: String,
    current_format_version: u8,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct ManifestFile {
    r#type: String,
    formatVersion: u8,
    id: Uuidv7,
    createdAt: IsoDateTime,
}

/// Creates a new Koli folder, which has:
/// - koli.json (likely to be deprecated with a db table later on)
/// - koli.db
pub fn create(folder_path: &str) -> Result<(), ArchiveError> {
    match fs::read_dir(folder_path) {
        Err(why) => Err(ArchiveError::IoError(why)),
        Ok(paths) => {
            // The dir has to be empty
            if paths.count() == 0 {
                create_manifest_file(folder_path)?;
                //TODO: 2. Initiliaze the db
                //TODO: 3. Done. Consider returning the path or something
                Ok(())
            } else {
                Err(ArchiveError::DirNotEmpty)
            }
        }
    }
}

pub fn open(folder_path: &str) {
    todo!()
}

fn koli_folder_files() -> KoliFolderFiles {
    KoliFolderFiles {
        manifest_file_name: String::from("koli.json"),
        database_file_name: String::from("koli.db"),
        account_dir_name: String::from("accounts"),
        current_format_version: 1,
    }
}

fn create_manifest_file(folder_path: &str) -> Result<(), ArchiveError> {
    let manifest_content = ManifestFile {
        r#type: String::from("koli"),
        formatVersion: 1,
        id: Uuid::now_v7(),
        createdAt: Utc::now(),
    };

    match serde_json::to_string(&manifest_content) {
        Err(e) => Err(ArchiveError::SerdeError(e)),
        Ok(x) => {
            fs::write(
                format!("{folder_path}{}", koli_folder_files().manifest_file_name),
                &x,
            )
            .unwrap();
            Ok(())
        }
    }
}

fn create_database() {
    todo!()
}
