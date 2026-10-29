//! The one way a session creates a task or an epic: in a project it names, with a description and a
//! first checklist of drafts; and the one way it files a task under an epic. The console's task
//! service in rekall-console stays the only path that edits, reviews or deletes one, and the only one
//! that can create a task with real steps. Also the view another task's `[[task:label]]` names a task by.

mod event;
mod model;
mod service;
mod view;

pub use model::created::Created;
pub use model::epic_joined::EpicJoined;
pub use service::epic_membership_service::EpicMembershipService;
pub use model::step_draft::StepDraft;
pub use service::task_authoring_service::TaskAuthoringService;
pub use model::task_brief::TaskBrief;
pub use view::task_reference_view::TaskReferenceView;
pub use event::task_stream_event::TaskStreamEvent;
pub use rekall_store::TaskKind;
