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

use rekall_common::RekallError;

/// Spring Data's `upper(x) = upper(?)`.
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    a.to_uppercase() == b.to_uppercase()
}

/// `LOWER(x) LIKE LOWER(CONCAT('%', term, '%'))`: whether `text` holds `term`, case folded.
pub fn contains_ignore_case(text: &str, term: &str) -> bool {
    text.to_lowercase().contains(&term.to_lowercase())
}

/// A derived query declared to return `Optional<T>` that found more than one row: Spring Data
/// threw `IncorrectResultSizeDataAccessException`, which nothing handled.
pub fn at_most_one<T>(mut rows: Vec<T>) -> Result<Option<T>, RekallError> {
    match rows.len() {
        0 => Ok(None),
        1 => Ok(rows.pop()),
        n => Err(RekallError::internal(
            "IncorrectResultSizeDataAccessException",
            format!("Query did not return a unique result: {n} results were returned"),
        )),
    }
}
