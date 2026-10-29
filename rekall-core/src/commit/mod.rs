//! Commits of a project's own folder: read from git (`GitLogReader`), made by git
//! (`GitCommitter`), inspected (`GitRepositoryInspector`), logged against a task or one of its
//! steps (`CommitReferenceService`), and made on a session's claim when the project asks for it
//! (`AutoCommitService`), under a message `CommitMessageGenerator` writes.

mod event;
mod model;
mod service;
mod view;
mod git;

pub use model::auto_commit_outcome::AutoCommitOutcome;
pub use service::auto_commit_service::AutoCommitService;
pub use model::auto_commit_status::AutoCommitStatus;
pub use model::commit_message_generator::{CommitMessageGenerator, BODY_WIDTH, DERIVED_BODY_MAX, SUBJECT_MAX};
pub use service::commit_reference_service::{CommitReferenceService, RECENT_LIMIT};
pub use event::commit_reference_stream_event::CommitReferenceStreamEvent;
pub use view::commit_reference_view::CommitReferenceView;
pub use git::{
    GitCommitter, GitLogReader, GitRepositoryInspector, LogEntry, LoggedCommit, PendingChange, PendingChangeKind,
    RepositoryStatus,
};
pub use view::recent_commit_view::RecentCommitView;
pub use model::subject::Subject;
