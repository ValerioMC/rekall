//! A task's thread as the console writes it: the person adds, edits and deletes feedback while it
//! is pending, starts a pass that takes every waiting entry, follows up an applied entry, discards a revision, and every saved change of an evaluated task's description is
//! tracked as a revision. A session answers entries through `rekall-core`.

mod command;
mod rule;
mod service;
mod view;

pub use command::add_feedback_command::AddFeedbackCommand;
pub use rule::revision_rule::{decide, is_evaluated, OpenRevision, RevisionDecision};
pub use service::feedback_service::FeedbackService;
pub use view::feedback_view::FeedbackView;
