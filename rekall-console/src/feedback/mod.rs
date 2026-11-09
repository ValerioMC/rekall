//! A task's revisions as the console writes them: every saved change of an evaluated task's
//! description is tracked as a revision, which the person points at the work or the plan, starts a
//! pass over every waiting one, or discards. A session answers revisions through `rekall-core`.

mod rule;
mod service;
mod view;

pub use rule::revision_rule::{decide, default_target, is_evaluated, OpenRevision, RevisionDecision};
pub use service::feedback_service::FeedbackService;
pub use view::feedback_view::FeedbackView;
