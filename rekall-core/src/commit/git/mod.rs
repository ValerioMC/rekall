//! The `git` process wrapper: one invocation in a folder, its exit code and both streams, and
//! the three components built on it. A hung or unrunnable git is reported, never waited on
//! forever; every failure is an `IllegalArgument` the caller turns into its own words.

mod git_command;
mod git_committer;
mod git_log_reader;
mod git_repository_inspector;
mod git_result;
mod log_entry;
mod logged_commit;
mod pending_change;
mod pending_change_kind;
mod repository_status;

pub use git_command::{display_path, run};
pub use git_committer::GitCommitter;
pub use git_log_reader::GitLogReader;
pub use git_repository_inspector::GitRepositoryInspector;
pub use git_result::GitResult;
pub use log_entry::LogEntry;
pub use logged_commit::LoggedCommit;
pub use pending_change::PendingChange;
pub use pending_change_kind::PendingChangeKind;
pub use repository_status::RepositoryStatus;
