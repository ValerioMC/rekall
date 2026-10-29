//! One module of query functions per entity, in a folder per domain and re-exported here. Every function takes any connection or
//! transaction, so a service reads inside the transaction it writes in.
//!
//! Case-insensitive comparisons are made in Rust rather than in SQL: SQLite's `LOWER`/`UPPER` and
//! `LIKE` fold ASCII only, and a company called "Ärzte" has to be found as "ärzte".

pub use crate::catalog::{repository::company_repository as company, repository::company_logo_image_repository as company_logo_image, repository::project_repository as project, repository::project_icon_image_repository as project_icon_image};
pub use crate::commit::repository::commit_reference_repository as commit_reference;
pub use crate::component::repository::project_component_repository as project_component;
pub use crate::connection::{repository::connection_access_repository as connection_access, repository::external_connection_repository as external_connection, repository::project_connection_repository as project_connection};
pub use crate::diagram::diagram_repository as diagram;
pub use crate::flow::{repository::step_link_repository as step_link, repository::step_visit_repository as step_visit};
pub use crate::graph::{repository::deployment_graph_repository as deployment_graph, repository::graph_route_repository as graph_route, repository::graph_step_repository as graph_step};
pub use crate::inventory::repository::project_dependency_repository as project_dependency;
pub use crate::note::repository::document_repository as document;
pub use crate::queue::repository::run_queue_repository as run_queue;
pub use crate::section::repository::project_section_repository as project_section;
pub use crate::skill::{repository::skill_placement_repository as skill_placement, repository::tag_skill_repository as tag_skill, repository::task_skill_repository as task_skill};
pub use crate::step::repository::task_step_repository as task_step;
pub use crate::tag::tag_repository as tag;
pub use crate::task::task_repository as task;
pub use crate::timeentry::repository::time_entry_repository as time_entry;
pub use crate::token::{repository::token_usage_repository as token_usage, repository::transcript_scan_repository as transcript_scan};
pub use crate::vulnerability::{repository::scan_scope_repository as scan_scope, repository::security_advisory_repository as security_advisory, repository::vulnerability_exposure_repository as vulnerability_exposure};
pub use crate::wrapup::wrapup_repository as wrapup;
pub use crate::query_support::{at_most_one, contains_ignore_case, equals_ignore_case};
pub use crate::stamped_update::update_if_changed;
