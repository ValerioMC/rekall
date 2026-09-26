//! The task-scoped mirror of the step line: walks a stepless task along
//! `OPEN -> RUNNING -> CLAIMED -> DONE` and pushes every move onto the step feed as a
//! `task-review` event. `session_running` and `claimed_by_wrapup` ride on existing signals;
//! `accept` and `send_back` are the console's alone, with no MCP tool. Every method is inert once
//! the task has a checklist.

mod task_review_event;
mod task_review_service;
mod task_review_view;

pub use task_review_event::TaskReviewEvent;
pub use task_review_service::TaskReviewService;
pub use task_review_view::TaskReviewView;
