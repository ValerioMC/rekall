//! The persistence layer: where the SQLite file is opened, the migrations that build its
//! schema, and the queries the services read and write with.
//!
//! The migrations keep the ids of the changesets of the database the Java build used, so a file
//! imported from it (`rekall-app/src/h2`) resumes at the right one.

pub mod connection;
pub mod migration;
pub mod repository;

pub use connection::{open, open_in_memory, open_temporary, Database, DATA_FILE_NAME};
