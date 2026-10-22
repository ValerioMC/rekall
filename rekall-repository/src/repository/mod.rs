//! One module of query functions per entity, in a folder per domain and re-exported here. Every function takes any connection or
//! transaction, so a service reads inside the transaction it writes in.
//!
//! Case-insensitive comparisons are made in Rust rather than in SQL: SQLite's `LOWER`/`UPPER` and
//! `LIKE` fold ASCII only, and a company called "Ärzte" has to be found as "ärzte".

pub mod catalog;
pub mod commit;
pub mod component;
pub mod connection;
pub mod diagram;
pub mod flow;
pub mod note;
pub mod queue;
pub mod section;
pub mod skill;
pub mod step;
pub mod tag;
pub mod task;
pub mod timeentry;
pub mod token;
pub mod wrapup;

mod query_support;
mod stamped_update;

pub use catalog::{company, company_logo_image, project, project_icon_image};
pub use commit::commit_reference;
pub use component::project_component;
pub use connection::{connection_access, external_connection, project_connection};
pub use flow::{step_link, step_visit};
pub use note::document;
pub use queue::run_queue;
pub use section::project_section;
pub use skill::{skill_placement, tag_skill, task_skill};
pub use step::task_step;
pub use timeentry::time_entry;
pub use token::{token_usage, transcript_scan};
pub use query_support::{at_most_one, contains_ignore_case, equals_ignore_case};
pub use stamped_update::update_if_changed;
