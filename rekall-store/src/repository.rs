//! One module of query functions per entity, in a folder per domain and re-exported here. Every function takes any connection or
//! transaction, so a service reads inside the transaction it writes in.
//!
//! Case-insensitive comparisons are made in Rust rather than in SQL: SQLite's `LOWER`/`UPPER` and
//! `LIKE` fold ASCII only, and a company called "Ärzte" has to be found as "ärzte".

pub use crate::catalog::{company_repository as company, company_logo_image_repository as company_logo_image, project_repository as project, project_icon_image_repository as project_icon_image};
pub use crate::commit::commit_reference_repository as commit_reference;
pub use crate::component::project_component_repository as project_component;
pub use crate::connection::{connection_access_repository as connection_access, external_connection_repository as external_connection, project_connection_repository as project_connection};
pub use crate::diagram::diagram_repository as diagram;
pub use crate::flow::{step_link_repository as step_link, step_visit_repository as step_visit};
pub use crate::graph::{deployment_graph_repository as deployment_graph, graph_route_repository as graph_route, graph_step_repository as graph_step};
pub use crate::inventory::project_dependency_repository as project_dependency;
pub use crate::note::document_repository as document;
pub use crate::queue::run_queue_repository as run_queue;
pub use crate::section::project_section_repository as project_section;
pub use crate::skill::{skill_placement_repository as skill_placement, tag_skill_repository as tag_skill, task_skill_repository as task_skill};
pub use crate::step::task_step_repository as task_step;
pub use crate::tag::tag_repository as tag;
pub use crate::task::task_repository as task;
pub use crate::timeentry::time_entry_repository as time_entry;
pub use crate::token::{token_usage_repository as token_usage, transcript_scan_repository as transcript_scan};
pub use crate::vulnerability::{scan_scope_repository as scan_scope, security_advisory_repository as security_advisory, vulnerability_exposure_repository as vulnerability_exposure};
pub use crate::wrapup::wrapup_repository as wrapup;
pub use crate::query_support::{at_most_one, contains_ignore_case, equals_ignore_case};
pub use crate::stamped_update::update_if_changed;
