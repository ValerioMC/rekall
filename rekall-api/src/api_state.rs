use std::path::PathBuf;
use std::sync::Arc;

use rekall_service::Services;

use crate::{service, StepEventStream};

/// What the handlers share.
#[derive(Clone)]
pub struct ApiState {
    pub services: Services,
    pub companies: service::CompanyService,
    pub projects: service::ProjectService,
    pub tasks: service::TaskService,
    pub documents: service::DocumentService,
    pub tags: service::TagService,
    pub diagrams: service::DiagramCatalogService,
    pub sources: service::SourceExcerptService,
    pub sections: service::ProjectSectionCatalogService,
    pub icons: service::ProjectIconService,
    pub restorer: service::RevisionRestoreService,
    pub directories: service::DirectoryListingService,
    pub stream: Arc<StepEventStream>,
}

impl ApiState {
    /// `user_home` is where the path picker starts.
    pub fn new(services: Services, stream: Arc<StepEventStream>, user_home: PathBuf) -> Self {
        let tasks = service::TaskService::new(&services);
        Self {
            documents: service::DocumentService::new(services.clone()),
            tags: service::TagService::new(services.clone()),
            diagrams: service::DiagramCatalogService::new(services.clone()),
            sources: service::SourceExcerptService::new(services.clone()),
            sections: service::ProjectSectionCatalogService::new(services.clone()),
            icons: service::ProjectIconService::new(services.clone()),
            restorer: service::RevisionRestoreService::new(services.clone(), tasks.clone()),
            directories: service::DirectoryListingService::new(user_home),
            companies: service::CompanyService::new(services.ctx.clone()),
            projects: service::ProjectService::new(services.ctx.clone(), services.repositories),
            tasks,
            services,
            stream,
        }
    }
}
