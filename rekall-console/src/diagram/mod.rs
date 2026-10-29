//! Stored diagrams and the source excerpts their code links open.

mod service;
mod view;

pub use service::diagram_catalog_service::DiagramCatalogService;
pub use service::source_excerpt_service::SourceExcerptService;
pub use view::source_excerpt_view::SourceExcerptView;
pub use view::source_line_view::SourceLineView;
