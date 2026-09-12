#[derive(Debug, PartialEq, Eq)]
pub(crate) struct MessageRow {
    pub(crate) id: String,
    pub(crate) account_id: String,
    pub(crate) platform: String,
    pub(crate) conversation_id: String,
    pub(crate) record_id: String,
    pub(crate) sender: String,
    pub(crate) recipient: Option<String>,
    pub(crate) text: Option<String>,
    pub(crate) created_at_ms: Option<i64>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ReactionRow {
    pub(crate) main_id: String,
    pub(crate) ordinal: i64,
    pub(crate) record_id: Option<String>,
    pub(crate) sender: Option<String>,
    pub(crate) reaction: String,
    pub(crate) created_at_ms: Option<i64>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct EditRow {
    pub(crate) main_id: String,
    pub(crate) ordinal: i64,
    pub(crate) text: String,
    pub(crate) created_at_ms: Option<i64>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FileAttachmentRow {
    pub(crate) main_id: String,
    pub(crate) ordinal: i64,
    pub(crate) file_rel_path: String,
    pub(crate) created_at_ms: Option<i64>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LinkAttachmentRow {
    pub(crate) main_id: String,
    pub(crate) ordinal: i64,
    pub(crate) url: String,
    pub(crate) created_at_ms: Option<i64>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct MessageImportRows {
    pub(crate) messages: Vec<MessageRow>,
    pub(crate) reactions: Vec<ReactionRow>,
    pub(crate) edits: Vec<EditRow>,
    pub(crate) file_attachments: Vec<FileAttachmentRow>,
    pub(crate) link_attachments: Vec<LinkAttachmentRow>,
}
