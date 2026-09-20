//! Contacts associate participant identities from imported datasets with people.
//!
//! A contact can be shared by identities from multiple accounts and datasets. Each
//! dataset type owns its assignment rules; message participant assignments are
//! implemented in [`messages`].

mod create;
mod messages;
mod repo;

pub mod models;

pub use messages::get_message_participant_contact;
