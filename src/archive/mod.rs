//! This file deals with Koli archive folder related operations.
//!
//! A Koli \[archive\] folder is a folder user chooses to save their data. Everything, from
//! the SQLite database to raw imports and media files are stored in this folder.
//! Users can create a new Koli folder, or choose an existing one.

pub mod create;
pub mod open;

pub mod model;
pub(crate) mod utils;
