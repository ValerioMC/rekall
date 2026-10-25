//! The schema: `baseline` builds it as release v0.2.x left it, and each later change is a
//! migration of its own, named by its id and applied in order.
//!
//! Types: ids and instants are TEXT (an `Id` and an `Instant` are stored as their text),
//! `VARCHAR(n)` carries a length SQLite does not enforce (the model does, see
//! `rekall_model::constraints`), booleans are 0/1.

mod baseline;
mod m031_project_section;
mod m032_project_icon_image;
mod m033_project_component;
mod m034_company_logo_image;
mod m035_mermaid_diagram;
mod m036_token_usage;
mod m037_skill_placement;
mod m038_skill_assignment;
mod m039_step_flow;
mod m040_flow_start_end;
mod m041_token_usage_day;
mod m042_step_max_visits;
mod m043_diagram_walkthrough;
mod m044_connection;
mod m045_connection_proxy;
mod m046_drop_task_revision;
mod m047_vulnerability;
mod m048_exposure_conclusion;
mod m049_deployment_graph;
mod m050_graph_step_place;
mod m051_binding_label;
mod m052_advisory_reach;
mod migration_support;
mod migrator;

pub use migrator::Migrator;

pub(crate) use baseline::{adopt as adopt_baseline, check as check_baseline};
pub(crate) use migration_support::{changeset, exec};
