//! Commits of a project's own folder: read from git (`GitLogReader`), made by git
//! (`GitCommitter`), inspected (`GitRepositoryInspector`), logged against a task or one of its
//! steps (`CommitReferenceService`), and made on a session's claim when the project asks for it
//! (`AutoCommitService`), under a message `CommitMessageGenerator` writes. `StepSnapshotService`
//! records which files were already modified when a step started. `TaskChangesService` reads what
//! a task changed: its logged commits' patches and the working tree.

mod event;
mod model;
mod service;
mod view;
mod git;

pub use model::auto_commit_outcome::AutoCommitOutcome;
pub use service::auto_commit_service::AutoCommitService;
pub use service::step_snapshot_service::StepSnapshotService;
pub use model::auto_commit_status::AutoCommitStatus;
pub use model::body_wrapping::BODY_WIDTH;
pub use model::commit_message_generator::{CommitMessageGenerator, SUBJECT_MAX};
pub use model::markdown_plain_text::DERIVED_BODY_MAX;
pub use service::commit_reference_service::{CommitReferenceService, RECENT_LIMIT};
pub use service::task_changes_service::TaskChangesService;
pub use event::commit_reference_stream_event::CommitReferenceStreamEvent;
pub use view::commit_reference_view::CommitReferenceView;
pub use git::{run as run_git, run_with_environment as run_git_with_environment, GitResult};
pub use git::{
    CommitTouch, GitCommitter, GitLogReader, GitRepositoryInspector, GitWorkingTreeReader, LogEntry, LoggedCommit, PendingChange, PendingChangeKind,
    RepositoryStatus,
};
pub use view::recent_commit_view::RecentCommitView;
pub use view::{ChangeKind, ChangedFileView, CommitChangesView, DiffHunkView, DiffLineKind, DiffLineView, RepositoryChangesView, TaskChangesView, WorkingTreeChangesView};
pub use model::repository_target::RepositoryTarget;
pub use model::subject::Subject;
