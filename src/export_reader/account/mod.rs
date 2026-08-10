//! An account in kolib is where data from an account of a platform is be stored in.
//!
//! For instance, a user can create an account with the name @my_old_acc for Twitter, in that account,
//! only pre-defined export/takeout files knowing to be coming from Twitter will be accepted.
//! When user requests to see the data from @my_old_acc, they will see direct messages, tweets,
//! following/followers list of that account.
//!
//! An account is a requirement when an export/takeout file from a platform is to be imported.

pub mod create;
pub mod models;
pub mod repo;
