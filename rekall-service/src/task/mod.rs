//! The one way a session creates a task: in a project it names, with a description and a first
//! checklist of drafts. The console's task service in rekall-api stays the only path that edits,
//! reviews or deletes one, and the only one that can create a task with real steps.

mod created;
mod step_draft;
mod task_authoring_service;
mod task_stream_event;

pub use created::Created;
pub use step_draft::StepDraft;
pub use task_authoring_service::TaskAuthoringService;
pub use task_stream_event::TaskStreamEvent;
