//! The console-only services: they write the catalog, delete diagrams or read the disk, so they
//! live here and not in `rekall-service`, out of `rekall-mcp`'s reach.

mod company_logo_service;
mod company_service;
mod diagram_catalog_service;
mod directory_listing_service;
mod document_service;
mod icon_raster;
mod project_component_catalog_service;
mod project_icon_service;
mod project_service;
mod project_section_catalog_service;
mod revision_restore_service;
mod snapshot;
mod source_excerpt_service;
mod tag_service;
mod task_service;

pub use company_logo_service::CompanyLogoService;
pub use company_service::CompanyService;
pub use diagram_catalog_service::DiagramCatalogService;
pub use directory_listing_service::{DirectoryListingService, ENTRY_LIMIT};
pub use document_service::DocumentService;
pub use icon_raster::{MASTER_SIDE, MAX_UPLOAD_BYTES};
pub use project_component_catalog_service::ProjectComponentCatalogService;
pub use project_icon_service::ProjectIconService;
pub use project_service::ProjectService;
pub use project_section_catalog_service::ProjectSectionCatalogService;
pub use revision_restore_service::RevisionRestoreService;
pub use snapshot::Snapshot;
pub use source_excerpt_service::SourceExcerptService;
pub use tag_service::TagService;
pub use task_service::TaskService;
