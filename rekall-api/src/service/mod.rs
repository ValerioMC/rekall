//! The console-only services: they write the catalog, so they live here and not in
//! `rekall-service`, out of `rekall-mcp`'s reach.

mod catalog;
mod document;
mod export;
mod graph;
mod revision_restore;
mod tag;

pub use catalog::CatalogService;
pub use document::DocumentService;
pub use export::ExportService;
pub use graph::Snapshot;
pub use revision_restore::RevisionRestoreService;
pub use tag::TagService;
