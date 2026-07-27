use regex::Regex;

use crate::error::ExportReaderError;

pub mod direct_messages;

const SUPPORTED_FILE_NAMES: &[&str] = &["direct_messages.js"];

#[allow(dead_code)]
pub struct TwitterImport {
    archive_dir: Option<String>,
    account: Option<String>, // TODO: Replace with Account struct
    file_name: Option<String>,
    file_content: Option<String>,
}

impl TwitterImport {
    pub fn new() -> Self {
        TwitterImport {
            archive_dir: None,
            account: None,
            file_name: None,
            file_content: None,
        }
    }

    pub fn file_name(mut self, file_name: &str) -> Result<Self, ExportReaderError> {
        if SUPPORTED_FILE_NAMES.contains(&file_name) {
            self.file_name = Some(file_name.to_string());
            Ok(self)
        } else {
            Err(ExportReaderError::InvalidOrUnsupportedFileName {
                platform: "Twitter/X".to_string(),
                file_name: file_name.to_string(),
            })
        }
    }

    pub fn file_content(mut self, file_content: &str) -> Self {
        self.file_content = Some(file_content.to_string());
        self
    }

    pub fn save_to_account(&self) -> Result<String, String> {
        self.convert_to_json().unwrap();

        todo!();
    }

    fn convert_to_json(&self) -> Result<String, ExportReaderError> {
        if let Some(f) = &self.file_content {
            let re = Regex::new(r"^[^=]*=\s*|;$").unwrap();
            let jsonized = re.replace_all(f.trim(), "");

            Ok(jsonized.to_string())
        } else {
            Err(ExportReaderError::FileContentNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::export_reader::platforms::twitter::TwitterImport;

    #[test]
    fn test_convertion() {
        let js_content = r#"
window.YTD.direct_messages.part0 = [
  {
    "dmConversation" : {
      "conversationId" : "123-456",
      "messages" : [
        {
          "messageCreate" : {
            "recipientId" : "456",
            "reactions" : [ ],
            "urls" : [ ],
            "text" : "Hey",
            "mediaUrls" : [ ],
            "senderId" : "123",
            "id" : "487584",
            "createdAt" : "2026-04-03T15:32:00.290Z",
            "editHistory" : [ ]
          }
        },
        {
          "messageCreate" : {
            "recipientId" : "456",
            "reactions" : [ ],
            "urls" : [ ],
            "text" : "Hi",
            "mediaUrls" : [ ],
            "senderId" : "123",
            "id" : "8384903",
            "createdAt" : "2026-04-03T16:20:40.678Z",
            "editHistory" : [ ]
          }
        }
        }
        }
        ];
        "#;

        let expected = r#"
[
  {
    "dmConversation" : {
      "conversationId" : "123-456",
      "messages" : [
        {
          "messageCreate" : {
            "recipientId" : "456",
            "reactions" : [ ],
            "urls" : [ ],
            "text" : "Hey",
            "mediaUrls" : [ ],
            "senderId" : "123",
            "id" : "487584",
            "createdAt" : "2026-04-03T15:32:00.290Z",
            "editHistory" : [ ]
          }
        },
        {
          "messageCreate" : {
            "recipientId" : "456",
            "reactions" : [ ],
            "urls" : [ ],
            "text" : "Hi",
            "mediaUrls" : [ ],
            "senderId" : "123",
            "id" : "8384903",
            "createdAt" : "2026-04-03T16:20:40.678Z",
            "editHistory" : [ ]
          }
        }
        }
        }
        ]
        "#;

        let import = TwitterImport::new().file_content(js_content);
        let jsonized = import.convert_to_json().unwrap();

        assert_eq!(jsonized.trim(), expected.trim())
    }
}
