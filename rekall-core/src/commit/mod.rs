//! Commits of a project's own folder: read from git (`GitLogReader`), made by git
//! (`GitCommitter`), inspected (`GitRepositoryInspector`), logged against a task or one of its
//! steps (`CommitReferenceService`), and made on a session's claim when the project asks for it
//! (`AutoCommitService`), under a message `CommitMessageGenerator` writes.

mod auto_commit_outcome;
mod auto_commit_service;
mod auto_commit_status;
mod commit_message_generator;
mod commit_reference_service;
mod commit_reference_stream_event;
mod commit_reference_view;
mod git;
mod recent_commit_view;
mod subject;

pub use auto_commit_outcome::AutoCommitOutcome;
pub use auto_commit_service::AutoCommitService;
pub use auto_commit_status::AutoCommitStatus;
pub use commit_message_generator::{CommitMessageGenerator, BODY_WIDTH, DERIVED_BODY_MAX, SUBJECT_MAX};
pub use commit_reference_service::{CommitReferenceService, RECENT_LIMIT};
pub use commit_reference_stream_event::CommitReferenceStreamEvent;
pub use commit_reference_view::CommitReferenceView;
pub use git::{
    GitCommitter, GitLogReader, GitRepositoryInspector, LogEntry, LoggedCommit, PendingChange, PendingChangeKind,
    RepositoryStatus,
};
pub use recent_commit_view::RecentCommitView;
pub use subject::Subject;
