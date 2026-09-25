//! The business logic. Everything the console and the MCP tools can do to the data goes through
//! a service here, and every service method that was `@Transactional` in Java opens (or joins) a
//! transaction here too, explicitly: a public method opens one, and its `*_in` twin runs inside a
//! transaction a caller already holds, which is how Spring's default propagation joined them.
//!
//! Events work as `@TransactionalEventListener(AFTER_COMMIT)` did: a service records them on the
//! transaction it runs in, and they reach the bus only once that transaction commits. A write
//! that rolls back announces nothing.

pub mod claude;
pub mod commit;
pub mod context;
pub mod events;
pub mod note;
pub mod review;
pub mod revision;
pub mod search;
pub mod step;
pub mod timeentry;
pub mod tx;
pub mod wrapup;

mod load;

use std::sync::Arc;

pub use events::{DomainEvent, EventBus};
pub use tx::Tx;

use rekall_common::Instant;
use sea_orm::DatabaseConnection;

/// What a clock-driven rule reads the time from. The application reads the system clock; a test
/// can fix it, the way the Java services took a `java.time.Clock`.
pub type Clock = Arc<dyn Fn() -> Instant + Send + Sync>;

pub fn system_clock() -> Clock {
    Arc::new(Instant::now)
}

/// The database and the event bus every service reads and writes through.
#[derive(Clone)]
pub struct Ctx {
    pub db: DatabaseConnection,
    pub events: EventBus,
    pub clock: Clock,
}

impl Ctx {
    pub fn new(db: DatabaseConnection, events: EventBus) -> Self {
        Self { db, events, clock: system_clock() }
    }

    pub fn now(&self) -> Instant {
        (self.clock)()
    }

    pub async fn read(&self) -> rekall_common::Result<Tx> {
        Tx::read(self).await
    }

    pub async fn write(&self) -> rekall_common::Result<Tx> {
        Tx::write(self).await
    }
}

/// Every service, built once over one context: what a controller or a tool is handed.
#[derive(Clone)]
pub struct Services {
    pub ctx: Ctx,
    pub steps: step::TaskStepService,
    pub review: review::TaskReviewService,
    pub revisions: revision::TaskRevisionService,
    pub wrapups: wrapup::WrapupService,
    pub notes: note::NoteService,
    pub time_entries: timeentry::TimeEntryService,
    pub context: context::ContextService,
    pub renderer: context::ContextRenderer,
    pub context_size: context::ContextSizeService,
    pub search: search::SearchService,
    pub commit_references: commit::CommitReferenceService,
    pub auto_commit: commit::AutoCommitService,
    pub repositories: commit::GitRepositoryInspector,
    pub task_work: claude::TaskWorkService,
    pub terminal_launch: claude::TerminalLaunchService,
}

impl Services {
    pub fn new(ctx: Ctx) -> Self {
        let revisions = revision::TaskRevisionService::new(ctx.clone());
        let review = review::TaskReviewService::new(ctx.clone());
        let commit_references = commit::CommitReferenceService::new(ctx.clone());
        let context = context::ContextService::new(ctx.clone());
        Self {
            steps: step::TaskStepService::new(ctx.clone()),
            wrapups: wrapup::WrapupService::new(ctx.clone(), review.clone(), revisions.clone()),
            notes: note::NoteService::new(ctx.clone()),
            time_entries: timeentry::TimeEntryService::new(ctx.clone()),
            renderer: context::ContextRenderer,
            context_size: context::ContextSizeService::new(ctx.clone(), context.clone()),
            search: search::SearchService::new(ctx.clone()),
            auto_commit: commit::AutoCommitService::new(ctx.clone(), commit_references.clone()),
            repositories: commit::GitRepositoryInspector,
            task_work: claude::TaskWorkService::new(ctx.clone()),
            terminal_launch: claude::TerminalLaunchService::new(ctx.clone()),
            context,
            commit_references,
            review,
            revisions,
            ctx,
        }
    }
}
