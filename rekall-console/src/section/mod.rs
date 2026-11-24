//! A project's sections.

mod command;
mod rule;
mod service;
mod view;

pub use service::project_section_catalog_service::ProjectSectionCatalogService;
pub use service::section_context_service::SectionContextService;
pub use service::section_layout_service::SectionLayoutService;
pub use command::project_section_command::ProjectSectionCommand;
pub use command::section_context_command::SectionContextCommand;
pub use command::section_layout_command::{ContextPlacementCommand, SectionLayoutCommand};
pub use view::section_layout_view::SectionLayoutView;
pub use view::section_write_outcome::SectionWriteOutcome;
