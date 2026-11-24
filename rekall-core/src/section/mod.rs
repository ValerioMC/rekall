//! A project's sections: titled markdown the console shows beside the blueprint, grouped in
//! contexts, and no task context ever loads. `ProjectSectionService` lists them and takes a
//! session's write, which adds a section (in the context it names, added when the project lacks it) or,
//! asked to replace, rewrites the body of the one with that title, and answers a session's read of
//! them, which is the only way a session sees them. A session can also reorganize them: name every
//! page once under the contexts it wants, and the pages move there. Everything else a person does to them (rename,
//! arrange, delete, and the contexts themselves) is the console's, in rekall-console.

mod event;
mod model;
pub mod rule;
mod service;
mod view;


pub use service::project_section_service::ProjectSectionService;
pub use view::project_section_view::ProjectSectionView;
pub use view::section_context_view::SectionContextView;
pub use event::section_stream_event::SectionStreamEvent;
pub use model::page_group::PageGroup;
pub use model::section_written::SectionWritten;
pub use model::sections_read::SectionsRead;
pub use model::sections_reorganized::SectionsReorganized;
