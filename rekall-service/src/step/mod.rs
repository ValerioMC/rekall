//! The step checklist: a task's lines of work, each on its line from draft to done.

mod service;
mod view;

pub use service::{Proposed, TaskStepService, PROPOSED_DRAFTS_MAX};
pub use view::{StepStreamEvent, TaskStepView};
