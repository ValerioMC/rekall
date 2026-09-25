//! The persistence layer: where the database file is opened, the migrations that build its
//! schema (one per Liquibase changeset, in the same order, under the same ids), and the queries
//! the Java repositories declared, one function per derived or `@Query` method.
//!
//! The database is SQLite. H2 had no Rust driver; SQLite is the same shape of thing, one file and
//! no server. Every changeset of `db.changelog-master.yaml` has a migration here that does the
//! same thing to a SQLite schema, including the ones whose tables were dropped again later, so a
//! schema built from nothing passes through exactly the states the H2 one did.

pub mod connection;
pub mod migration;
pub mod repository;

pub use connection::{open, open_in_memory, open_temporary, Database, DATA_FILE_NAME};
