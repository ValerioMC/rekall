//! The REST API the console calls: one router per Java controller, the request and response
//! shapes of `ApiDtos`, the console-only services that write the catalog (`CatalogService`,
//! `DocumentService`, `TagService`, `ExportService`, `RevisionRestoreService`), the folder
//! listing behind the path picker, and the one Server-Sent Events feed. `rekall-mcp` never sees any of it, which is what keeps the MCP write
//! surface narrow.

pub mod controller;
pub mod dto;
pub mod error;
pub mod extract;
pub mod service;
pub mod stream;

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use rekall_service::Services;

pub use error::{ApiError, ApiResult};
pub use stream::StepEventStream;

/// What the handlers share.
#[derive(Clone)]
pub struct ApiState {
    pub services: Services,
    pub catalog: service::CatalogService,
    pub documents: service::DocumentService,
    pub tags: service::TagService,
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
            export: service::ExportService::new(services.clone()),
            restorer: service::RevisionRestoreService::new(services.clone(), catalog.clone()),
            directories: service::DirectoryListingService::new(user_home),
            catalog,
            services,
            stream,
        }
    }
}

/// Every `/api` route this module owns.
pub fn router(state: ApiState) -> Router {
    controller::routes()
        .route_layer(axum::middleware::from_fn(extract::uuid_path_params))
        .with_state(state)
}
