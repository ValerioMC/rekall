//! A project's components, as a context and the console read them. Only the console writes them
//! (rekall-api): a session reads which folders are parts of the project and never edits the list.

mod project_component_view;

pub use project_component_view::ProjectComponentView;
