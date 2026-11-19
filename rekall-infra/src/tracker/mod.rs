//! The task manager a project writes its tasks to, over the REST API of GitLab, GitHub or Jira, the
//! runner that writes a task each time its title or description changes, and the pull requests of
//! its branches on GitHub or GitLab.

mod connection_pull_request_host;
mod github_pull_request_host;
mod gitlab_pull_request_host;
mod http_task_tracker;
mod task_tracker_runner;

pub use connection_pull_request_host::ConnectionPullRequestHost;
pub use github_pull_request_host::GithubPullRequestHost;
pub use gitlab_pull_request_host::GitlabPullRequestHost;
pub use http_task_tracker::HttpTaskTracker;
pub use task_tracker_runner::TaskTrackerRunner;
