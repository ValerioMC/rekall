//! A person's side of the decisions of a task: reading them as the console shows them, and editing,
//! adding or removing one by hand. A session writes through `rekall-core`. A record a person writes
//! or edits is marked as theirs and the context says so.

mod command;
mod service;
mod view;

pub use command::add_decision::AddDecision;
pub use command::edit_decision::EditDecision;
pub use service::task_decision_service::TaskDecisionService;
pub use view::task_decisions_view::TaskDecisionsView;
