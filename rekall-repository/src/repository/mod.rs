//! One module of query functions per entity. Every function takes any connection or
//! transaction, so a service reads inside the transaction it writes in.
//!
//! Case-insensitive comparisons are made in Rust rather than in SQL: SQLite's `LOWER`/`UPPER` and
//! `LIKE` fold ASCII only, and a company called "Ärzte" has to be found as "ärzte".

pub mod commit_reference;
pub mod company;
pub mod diagram;
pub mod document;
pub mod project;
pub mod project_icon_image;
pub mod project_section;
pub mod run_queue;
pub mod tag;
pub mod task;
pub mod task_revision;
pub mod task_step;
pub mod time_entry;
pub mod wrapup;

mod query_support;
mod stamped_update;

pub use query_support::{at_most_one, contains_ignore_case, equals_ignore_case};
pub use stamped_update::update_if_changed;
