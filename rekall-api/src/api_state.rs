use std::path::PathBuf;
use std::sync::Arc;

use rekall_service::commit::CommitReferenceService;
use rekall_service::context::ContextSizeService;
use rekall_service::diagram::DiagramService;
use rekall_service::revision::TaskRevisionService;
use rekall_service::search::SearchService;
use rekall_service::section::ProjectSectionService;
use rekall_service::step::TaskStepService;
use rekall_service::timeentry::TimeEntryService;
use rekall_service::wrapup::WrapupService;
use rekall_service::Services;

use crate::{service, StepEventStream};

/// What the handlers share: each service a controller calls, and nothing it does not.
#[derive(Clone)]
pub struct ApiState {
    pub steps: TaskStepService,
    pub wrapups: WrapupService,
    pub time_entries: TimeEntryService,
    pub revisions: TaskRevisionService,
    pub commit_references: CommitReferenceService,
    pub context_size: ContextSizeService,
    pub search: SearchService,
    pub diagrams: DiagramService,
    pub sections: ProjectSectionService,
    pub companies: service::CompanyService,
    pub projects: service::ProjectService,
    pub tasks: service::TaskService,
    pub documents: service::DocumentService,
    pub tags: service::TagService,
    pub diagram_catalog: service::DiagramCatalogService,
    pub sources: service::SourceExcerptService,
    pub section_catalog: service::ProjectSectionCatalogService,
    pub component_catalog: service::ProjectComponentCatalogService,
    pub icons: service::ProjectIconService,
    pub restorer: service::RevisionRestoreService,
    pub directories: service::DirectoryListingService,
    pub stream: Arc<StepEventStream>,
}

impl ApiState {
    /// The composition root of the API: `services` is taken apart here, so each controller
    /// reaches only the services it calls. `user_home` is where the path picker starts.
    pub fn new(services: Services, stream: Arc<StepEventStream>, user_home: PathBuf) -> Self {
        let ctx = services.ctx.clone();
        let tasks = service::TaskService::new(
            ctx.clone(),
            services.review.clone(),
            services.revisions.clone(),
            services.time_entries.clone(),
        );
        let restorer = service::RevisionRestoreService::new(
            ctx.clone(),
            services.revisions.clone(),
            services.wrapups.clone(),
            tasks.clone(),
        );
        Self {
            companies: service::CompanyService::new(ctx.clone()),
            projects: service::ProjectService::new(ctx.clone(), services.repositories),
            tasks,
            documents: service::DocumentService::new(ctx.clone()),
            tags: service::TagService::new(ctx.clone()),
            diagram_catalog: service::DiagramCatalogService::new(ctx.clone()),
            sources: service::SourceExcerptService::new(ctx.clone()),
            section_catalog: service::ProjectSectionCatalogService::new(ctx.clone()),
            component_catalog: service::ProjectComponentCatalogService::new(ctx.clone()),
            icons: service::ProjectIconService::new(ctx),
            restorer,
            directories: service::DirectoryListingService::new(user_home),
            steps: services.steps,
            wrapups: services.wrapups,
            time_entries: services.time_entries,
            revisions: services.revisions,
            commit_references: services.commit_references,
            context_size: services.context_size,
            search: services.search,
            diagrams: services.diagrams,
            sections: services.sections,
            stream,
        }
    }
}
