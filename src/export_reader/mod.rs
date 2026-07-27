use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{error::ExportReaderError, export_reader::account::Account};

pub mod account;
pub mod platforms;

/// ExportFile struct and its implementation is meant to be used as a field in structs of each
/// export file module.
/// # Example
/// ```
/// pub struct DirectMessagesImporter {
///     account: Option<Account>,
///     file: Option<ExportFile>,
///     platform: Platform,
/// }
/// ```
/// This helps streamlining how files should be handled. Only path is required to be provided, and
/// it handles the rest.
pub struct ExportFile {
    name: String,
    content: String,
    path: PathBuf,
}
impl ExportFile {
    pub fn new(mut self, file_path: PathBuf) -> Result<Self, std::io::Error> {
        self.name = Path::new(&file_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        self.content = fs::read_to_string(&file_path).unwrap();
        self.path = file_path;

        Ok(self)
    }

    /// Getters
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn path(self) -> PathBuf {
        self.path
    }
}

/// To be implemented by importer structs of file name modules
trait ExportParser {
    type DbRecord;

    fn new() -> Self;

    fn set_account(self, account: Account) -> Self;
    fn set_file(self, file: ExportFile) -> Self;

    /// Each export file module should have their valid file name(s) in them, and implement this
    /// function to check if the imported file is being used with the correct module.
    fn is_file_name_valid(&self) -> Result<bool, ExportReaderError>;

    /// Validate the file content, so we know it can be imported.
    /// There can be different ways of validation depending on the file type or the export.
    /// For instance, Twitter/X provides .js files, which are just a JS variable with a JSON content.
    /// Thus it can be converted to JSON, and can be validated with a JSON schema.
    fn is_file_content_valid(&self) -> bool;

    fn create_db_record(&self) -> Result<Vec<Self::DbRecord>, ()>;
}
