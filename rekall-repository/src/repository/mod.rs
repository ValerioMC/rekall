//! The queries the Java repositories declared, one function per derived or `@Query` method, named
//! after it. Every function takes any connection or transaction, so a service reads inside the
//! transaction it writes in.
//!
//! `IgnoreCase` comparisons are made here in Rust rather than in SQL: SQLite's `LOWER`/`UPPER`
//! and `LIKE` fold ASCII only, where H2 folded all of Unicode, and a company called "Ärzte" has
//! to be found as "ärzte" exactly as before.

pub mod commit_reference;
pub mod company;
pub mod document;
pub mod project;
pub mod run_queue;
pub mod tag;
pub mod task;
pub mod task_revision;
pub mod task_step;
pub mod time_entry;
pub mod wrapup;

mod query_support;

pub use query_support::{at_most_one, contains_ignore_case, equals_ignore_case};
