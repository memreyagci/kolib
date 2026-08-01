use chrono::{DateTime, Utc};
use strum::Display;
use uuid::Uuid;

pub type Uuidv7 = Uuid;
pub type IsoDateTime = DateTime<Utc>;

/// List of supported platforms. strum crate automatically implements functions necessary to get
/// enum field from string and vice versa.
#[derive(Debug, PartialEq, Clone, Copy, Display, strum::EnumString, strum::AsRefStr)]
#[strum(serialize_all = "lowercase")]
#[strum(ascii_case_insensitive)]
pub enum Platform {
    Twitter,
}
