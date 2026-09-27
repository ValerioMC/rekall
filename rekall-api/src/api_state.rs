use std::path::PathBuf;
use std::sync::Arc;

use rekall_service::Services;

use crate::{service, StepEventStream};

/// What the handlers share.
#[derive(Clone)]
pub struct ApiState {
    pub services: Services,
    pub catalog: service::CatalogService,
    pub documents: service::DocumentService,
    pub tags: service::TagService,
    pub diagrams: service::DiagramCatalogService,
    pub sources: service::SourceExcerptService,
    pub export: service::ExportService,
    pub restorer: service::RevisionRestoreService,
    pub directories: service::DirectoryListingService,
    pub stream: Arc<StepEventStream>,
}

impl ApiState {
    /// `user_home` is where the path picker starts: `System.getProperty("user.home")`.
    pub fn new(services: Services, stream: Arc<StepEventStream>, user_home: PathBuf) -> Self {
        let catalog = service::CatalogService::new(services.clone());
        Self {
            documents: service::DocumentService::new(services.clone()),
            tags: service::TagService::new(services.clone()),
            diagrams: service::DiagramCatalogService::new(services.clone()),
            sources: service::SourceExcerptService::new(services.clone()),
            export: service::ExportService::new(services.clone()),
            restorer: service::RevisionRestoreService::new(services.clone(), catalog.clone()),
            directories: service::DirectoryListingService::new(user_home),
            catalog,
            services,
            stream,
        }
    }
}
