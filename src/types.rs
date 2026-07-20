use chrono::{DateTime, Utc};
use uuid::Uuid;

pub type Uuidv7 = Uuid;
pub type IsoDateTime = DateTime<Utc>;

pub enum Platform {
    Twitter,
}
