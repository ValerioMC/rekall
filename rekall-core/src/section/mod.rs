//! A project's sections: titled markdown the console shows beside the blueprint and no context
//! ever loads. `ProjectSectionService` lists them and takes a session's write, which adds a
//! section or, asked to replace, rewrites the body of the one with that title, and answers a session's
//! read of them, which is the only way a session sees them. Everything else a
//! person does to them (rename, reorder, delete) is the console's, in rekall-console.

mod event;
mod model;
pub mod rule;
mod service;
mod view;


pub use service::project_section_service::ProjectSectionService;
pub use view::project_section_view::ProjectSectionView;
pub use event::section_stream_event::SectionStreamEvent;
pub use model::section_written::SectionWritten;
pub use model::sections_read::SectionsRead;
