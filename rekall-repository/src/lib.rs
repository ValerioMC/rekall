//! The persistence layer: where the SQLite file is opened, the migrations that build its
//! schema, and the queries the services read and write with.
//!
//! The schema starts from a baseline (the state release v0.2.x left), with each later change a
//! migration on top. A v0.2.x file is moved onto the baseline when it is opened.

pub mod connection;
pub mod migration;
pub mod repository;

pub use connection::{check_openable, open, open_in_memory, open_temporary, Database, DATA_FILE_NAME};
