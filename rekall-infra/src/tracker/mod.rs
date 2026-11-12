//! The task manager a project writes its tasks to, over the REST API of GitLab, GitHub or Jira, the
//! runner that writes a task each time its title or description changes, and the pull requests of
//! its branches on GitHub.

mod github_pull_request_host;
mod http_task_tracker;
mod task_tracker_runner;

pub use github_pull_request_host::GithubPullRequestHost;
pub use http_task_tracker::HttpTaskTracker;
pub use task_tracker_runner::TaskTrackerRunner;
