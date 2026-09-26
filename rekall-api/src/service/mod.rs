//! The console-only services: they write the catalog, so they live here and not in
//! `rekall-service`, out of `rekall-mcp`'s reach.

mod catalog_service;
mod directory_listing_service;
mod document_service;
mod export_service;
mod revision_restore_service;
mod snapshot;
mod tag_service;

pub use catalog_service::CatalogService;
pub use directory_listing_service::{DirectoryListingService, ENTRY_LIMIT};
pub use document_service::DocumentService;
pub use export_service::ExportService;
pub use revision_restore_service::RevisionRestoreService;
pub use snapshot::Snapshot;
pub use tag_service::TagService;
