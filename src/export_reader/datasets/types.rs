use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, strum::EnumString, strum::AsRefStr)]
#[strum(serialize_all = "lowercase")]
#[strum(ascii_case_insensitive)]
pub enum DatasetType {
    Messages,
}
