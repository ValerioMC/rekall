use crate::{claude, commit, context, diagram, flow, note, review, revision, search, section, step, task, timeentry, token, wrapup, Ctx};

/// Every service, built once over one context: what a controller or a tool is handed.
#[derive(Clone)]
pub struct Services {
    pub ctx: Ctx,
    pub steps: step::TaskStepService,
    pub task_authoring: task::TaskAuthoringService,
    pub step_links: flow::StepLinkService,
    pub flow: flow::FlowService,
    pub review: review::TaskReviewService,
    pub revisions: revision::TaskRevisionService,
    pub wrapups: wrapup::WrapupService,
    pub notes: note::NoteService,
    pub diagrams: diagram::DiagramService,
    pub sections: section::ProjectSectionService,
    pub time_entries: timeentry::TimeEntryService,
    pub token_usage: token::TokenUsageService,
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
        let steps = step::TaskStepService::new(ctx.clone(), time_entries.clone());
        let step_links = flow::StepLinkService::new(ctx.clone(), steps.clone());
        let flow = flow::FlowService::new(ctx.clone(), steps.clone(), step_links.clone(), flow::StepVisitService::new(ctx.clone()));
        Self {
            task_authoring: task::TaskAuthoringService::new(ctx.clone(), steps.clone()),
            steps,
            step_links,
            flow,
            wrapups: wrapup::WrapupService::new(ctx.clone(), review.clone(), revisions.clone()),
            notes: note::NoteService::new(ctx.clone()),
            diagrams: diagram::DiagramService::new(ctx.clone()),
            sections: section::ProjectSectionService::new(ctx.clone()),
            renderer: context::ContextRenderer,
            context_size: context::ContextSizeService::new(ctx.clone(), context.clone()),
            search: search::SearchService::new(ctx.clone()),
            auto_commit: commit::AutoCommitService::new(ctx.clone(), commit_references.clone()),
            repositories: commit::GitRepositoryInspector,
            task_work: claude::TaskWorkService::new(ctx.clone()),
            terminal_launch: claude::TerminalLaunchService::new(ctx.clone()),
            token_usage: token::TokenUsageService::new(ctx.clone()),
            context,
            commit_references,
            review,
            revisions,
            time_entries,
            ctx,
        }
    }
}
