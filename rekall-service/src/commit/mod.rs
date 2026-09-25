//! Commits of a project's own folder: read from git (`GitLogReader`), made by git
//! (`GitCommitter`), inspected (`GitRepositoryInspector`), logged against a task or one of its
//! steps (`CommitReferenceService`), and made on a session's claim when the project asks for it
//! (`AutoCommitService`), under a message `CommitMessageGenerator` writes.

mod auto_commit;
mod git;
mod message;
mod reference;

pub use auto_commit::{AutoCommitOutcome, AutoCommitService, AutoCommitStatus};
pub use git::{GitCommitter, GitLogReader, GitRepositoryInspector, LogEntry, LoggedCommit, PendingChange, PendingChangeKind, RepositoryStatus};
pub use message::{CommitMessageGenerator, Subject, BODY_WIDTH, DERIVED_BODY_MAX, SUBJECT_MAX};
pub use reference::{CommitReferenceService, CommitReferenceStreamEvent, CommitReferenceView, RecentCommitView, RECENT_LIMIT};
