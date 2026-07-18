//! This file deals with Koli archive folder related operations.
//!
//! A Koli \[archive\] folder is a folder user chooses to save their data. Everything, from
//! the sqlite database to raw imports and media files are stored in this folder.
//! Users can create a new Koli folder, or choose an existing one.

pub mod create;
pub use create::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

fn create_database() {
    todo!()
}
