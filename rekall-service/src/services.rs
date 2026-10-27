use crate::{claude, commit, context, diagram, flow, inventory, note, review, search, section, step, task, timeentry, token, vulnerability, wrapup, Ctx};

/// Every service, built once over one context: what a controller or a tool is handed.
#[derive(Clone)]
pub struct Services {
    pub ctx: Ctx,
    pub steps: step::TaskStepService,
    pub task_authoring: task::TaskAuthoringService,
    pub epic_membership: task::EpicMembershipService,
    pub step_links: flow::StepLinkService,
    pub flow: flow::FlowService,
    pub flow_proposals: flow::FlowProposalService,
    pub review: review::TaskReviewService,
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
    pub inventory: inventory::DependencyInventoryService,
    pub inventory_recording: vulnerability::InventoryRecordingService,
    pub advisories: vulnerability::SecurityAdvisoryService,
    pub exposures: vulnerability::VulnerabilityExposureService,
}

impl Services {
    pub fn new(ctx: Ctx) -> Self {
        let time_entries = timeentry::TimeEntryService::new(ctx.clone());
        let review = review::TaskReviewService::new(ctx.clone(), time_entries.clone());
        let commit_references = commit::CommitReferenceService::new(ctx.clone());
        let context = context::ContextService::new(ctx.clone());
        let steps = step::TaskStepService::new(ctx.clone(), time_entries.clone());
        let step_links = flow::StepLinkService::new(ctx.clone(), steps.clone());
        let flow = flow::FlowService::new(ctx.clone(), steps.clone(), step_links.clone(), flow::StepVisitService::new(ctx.clone()));
        let flow_proposals = flow::FlowProposalService::new(ctx.clone(), steps.clone(), step_links.clone());
        let inventory = inventory::DependencyInventoryService::new(ctx.clone());
        let exposures = vulnerability::VulnerabilityExposureService::new(ctx.clone(), commit::GitRepositoryInspector);
        Self {
            task_authoring: task::TaskAuthoringService::new(ctx.clone(), steps.clone()),
            epic_membership: task::EpicMembershipService::new(ctx.clone()),
            steps,
            step_links,
            flow,
            flow_proposals,
            wrapups: wrapup::WrapupService::new(ctx.clone(), review.clone()),
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
            inventory_recording: vulnerability::InventoryRecordingService::new(ctx.clone(), inventory.clone(), exposures.clone()),
            advisories: vulnerability::SecurityAdvisoryService::new(ctx.clone()),
            inventory,
            exposures,
            context,
            commit_references,
            review,
            time_entries,
            ctx,
        }
    }
}
