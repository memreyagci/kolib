use chrono::{DateTime, Utc, format::StrftimeItems};
use strum::Display;
use uuid::Uuid;

use crate::error::TimestampError;

pub type Uuidv7 = Uuid;
pub type IsoDateTime = DateTime<Utc>;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    pub const fn from_milliseconds(milliseconds: i64) -> Self {
        Self(milliseconds)
    }

    pub const fn milliseconds(self) -> i64 {
        self.0
    }

    pub fn to_datetime(self) -> Result<DateTime<Utc>, TimestampError> {
        DateTime::from_timestamp_millis(self.0).ok_or(TimestampError::OutOfRange {
            milliseconds: self.0,
        })
    }

    /// Formats this timestamp using Chrono's `strftime` syntax.
    pub fn format(self, format: &str) -> Result<String, TimestampError> {
        let items = StrftimeItems::new(format).parse()?;

        Ok(self
            .to_datetime()?
            .format_with_items(items.iter())
            .to_string())
    }
}

impl From<i64> for Timestamp {
    fn from(milliseconds: i64) -> Self {
        Self::from_milliseconds(milliseconds)
    }
}

/// List of supported platforms. strum crate automatically implements functions necessary to get
/// enum field from string and vice versa.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, strum::EnumString, strum::AsRefStr)]
#[strum(serialize_all = "lowercase")]
#[strum(ascii_case_insensitive)]
pub enum Platform {
    Twitter,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::Timestamp;

    #[test]
    fn converts_and_formats_timestamp() {
        let timestamp = Timestamp::from_milliseconds(1_788_213_960_008);

        assert_eq!(timestamp.milliseconds(), 1_788_213_960_008);
        assert_eq!(
            timestamp
                .format("%d/%m/%Y %H:%M:%S")
                .expect("formatting a valid timestamp should succeed"),
            "31/08/2026 22:06:00"
        );
    }

    #[test]
    fn rejects_out_of_range_timestamp_conversion() {
        let timestamp = Timestamp::from_milliseconds(i64::MAX);

        assert!(timestamp.to_datetime().is_err());
    }
}
