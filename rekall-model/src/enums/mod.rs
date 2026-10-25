//! The enumerations, stored and written on the wire by their upper-case name.

#[macro_use]
mod string_enum;

mod advisory_severity;
mod connection_kind;
mod document_context_mode;
mod ecosystem;
mod exposure_verdict;
mod fix_scope;
mod project_status;
mod run_queue_item_state;
mod run_queue_state;
mod task_status;
mod task_step_state;
mod wrapup_author;

pub use advisory_severity::AdvisorySeverity;
pub use connection_kind::ConnectionKind;
pub use document_context_mode::DocumentContextMode;
pub use ecosystem::Ecosystem;
pub use exposure_verdict::ExposureVerdict;
pub use fix_scope::FixScope;
pub use project_status::ProjectStatus;
pub use run_queue_item_state::RunQueueItemState;
pub use run_queue_state::RunQueueState;
pub use task_status::TaskStatus;
pub use task_step_state::TaskStepState;
pub use wrapup_author::WrapupAuthor;
