use crate::{claude, commit, context, diagram, note, review, revision, search, step, timeentry, wrapup, Ctx};

/// Every service, built once over one context: what a controller or a tool is handed.
#[derive(Clone)]
pub struct Services {
    pub ctx: Ctx,
    pub steps: step::TaskStepService,
    pub review: review::TaskReviewService,
    pub revisions: revision::TaskRevisionService,
    pub wrapups: wrapup::WrapupService,
    pub notes: note::NoteService,
    pub diagrams: diagram::DiagramService,
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
        let time_entries = timeentry::TimeEntryService::new(ctx.clone());
        let review = review::TaskReviewService::new(ctx.clone(), time_entries.clone());
        let commit_references = commit::CommitReferenceService::new(ctx.clone());
        let context = context::ContextService::new(ctx.clone());
        Self {
            steps: step::TaskStepService::new(ctx.clone(), time_entries.clone()),
            wrapups: wrapup::WrapupService::new(ctx.clone(), review.clone(), revisions.clone()),
            notes: note::NoteService::new(ctx.clone()),
            diagrams: diagram::DiagramService::new(ctx.clone()),
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
            time_entries,
            ctx,
        }
    }
}
