//! The enumerations, stored by name (`@Enumerated(EnumType.STRING)`) and written on the wire by
//! name, as Jackson wrote a Java enum.

#[macro_use]
mod string_enum;

mod document_context_mode;
mod project_status;
mod revision_kind;
mod run_queue_item_state;
mod run_queue_state;
mod task_status;
mod task_step_state;
mod wrapup_author;

pub use document_context_mode::DocumentContextMode;
pub use project_status::ProjectStatus;
pub use revision_kind::RevisionKind;
pub use run_queue_item_state::RunQueueItemState;
pub use run_queue_state::RunQueueState;
pub use task_status::TaskStatus;
pub use task_step_state::TaskStepState;
pub use wrapup_author::WrapupAuthor;
