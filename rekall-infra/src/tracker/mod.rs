//! The task manager a project writes its tasks to, over the REST API of GitLab, GitHub or Jira, and
//! the runner that writes a task each time its title or description changes.

mod http_task_tracker;
mod task_tracker_runner;

pub use http_task_tracker::HttpTaskTracker;
pub use task_tracker_runner::TaskTrackerRunner;
