use std::path::PathBuf;
use std::sync::Arc;

use rekall_service::commit::CommitReferenceService;
use rekall_service::connection::SecretStore;
use rekall_service::context::ContextSizeService;
use rekall_service::diagram::DiagramService;
use rekall_service::flow::{FlowService, StepLinkService};
use rekall_service::revision::TaskRevisionService;
use rekall_service::search::SearchService;
use rekall_service::section::ProjectSectionService;
use rekall_service::step::TaskStepService;
use rekall_service::timeentry::TimeEntryService;
use rekall_service::token::TokenUsageService;
use rekall_service::wrapup::WrapupService;
use rekall_service::Services;

use crate::{catalog, diagram, filesystem, note, revision, section, component, skill, tag, StepEventStream};

/// What the handlers share: each service a controller calls, and nothing it does not.
#[derive(Clone)]
pub struct ApiState {
    pub steps: TaskStepService,
    pub step_links: StepLinkService,
    pub flow: FlowService,
    pub wrapups: WrapupService,
    pub time_entries: TimeEntryService,
    pub token_usage: TokenUsageService,
    pub revisions: TaskRevisionService,
    pub commit_references: CommitReferenceService,
    pub context_size: ContextSizeService,
    pub search: SearchService,
    pub diagrams: DiagramService,
    pub sections: ProjectSectionService,
    pub companies: catalog::CompanyService,
    pub logos: catalog::CompanyLogoService,
    pub projects: catalog::ProjectService,
    pub tasks: catalog::TaskService,
    pub documents: note::DocumentService,
    pub tags: tag::TagService,
    pub skill_assignments: skill::SkillAssignmentService,
    pub diagram_catalog: diagram::DiagramCatalogService,
    pub sources: diagram::SourceExcerptService,
    pub section_catalog: section::ProjectSectionCatalogService,
    pub component_catalog: component::ProjectComponentCatalogService,
    pub icons: catalog::ProjectIconService,
    pub restorer: revision::RevisionRestoreService,
    pub directories: filesystem::DirectoryListingService,
    pub stream: Arc<StepEventStream>,
    /// Connection credentials; handed in by `rekall-app`, never part of `Services`.
    pub secrets: Arc<dyn SecretStore>,
}

impl ApiState {
    /// The composition root of the API: `services` is taken apart here, so each controller
    /// reaches only the services it calls. `user_home` is where the path picker starts.
    pub fn new(services: Services, stream: Arc<StepEventStream>, user_home: PathBuf, secrets: Arc<dyn SecretStore>) -> Self {
        let ctx = services.ctx.clone();
        let tasks = catalog::TaskService::new(
            ctx.clone(),
            services.review.clone(),
            services.revisions.clone(),
            services.time_entries.clone(),
        );
        let restorer = revision::RevisionRestoreService::new(
            ctx.clone(),
            services.revisions.clone(),
            services.wrapups.clone(),
            tasks.clone(),
        );
        Self {
            companies: catalog::CompanyService::new(ctx.clone()),
            logos: catalog::CompanyLogoService::new(ctx.clone()),
            projects: catalog::ProjectService::new(ctx.clone(), services.repositories),
            tasks,
            documents: note::DocumentService::new(ctx.clone()),
            tags: tag::TagService::new(ctx.clone()),
            skill_assignments: skill::SkillAssignmentService::new(ctx.clone()),
            diagram_catalog: diagram::DiagramCatalogService::new(ctx.clone()),
            sources: diagram::SourceExcerptService::new(ctx.clone()),
            section_catalog: section::ProjectSectionCatalogService::new(ctx.clone()),
            component_catalog: component::ProjectComponentCatalogService::new(ctx.clone()),
            icons: catalog::ProjectIconService::new(ctx),
            restorer,
            directories: filesystem::DirectoryListingService::new(user_home),
            steps: services.steps,
            step_links: services.step_links,
            flow: services.flow,
            wrapups: services.wrapups,
            time_entries: services.time_entries,
            token_usage: services.token_usage,
            revisions: services.revisions,
            commit_references: services.commit_references,
            context_size: services.context_size,
            search: services.search,
            diagrams: services.diagrams,
            sections: services.sections,
            stream,
            secrets,
        }
    }
}
