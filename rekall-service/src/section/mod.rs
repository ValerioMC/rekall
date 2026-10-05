//! A project's sections: titled markdown the console shows beside the blueprint and no context
//! ever loads. `ProjectSectionService` lists them and takes a session's write, which adds a
//! section or, asked to replace, rewrites the body of the one with that title. Everything else a
//! person does to them (rename, reorder, delete) is the console's, in rekall-api.

mod project_section_service;
mod project_section_view;
mod section_stream_event;
mod section_written;

pub mod rules;

pub use project_section_service::ProjectSectionService;
pub use project_section_view::ProjectSectionView;
pub use section_stream_event::SectionStreamEvent;
pub use section_written::SectionWritten;
