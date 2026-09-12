#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::AsRefStr)]
pub(crate) enum SupportedFile {
    #[strum(serialize = "direct-messages.js")]
    DirectMessages,
}

impl SupportedFile {
    pub(crate) const fn media_directory(self) -> Option<&'static str> {
        match self {
            Self::DirectMessages => Some("direct_messages_media"),
        }
    }
}
