//! A project's components, as a context and the console read them. Only the console writes them
//! (rekall-console): a session reads which folders are parts of the project and never edits the list.

mod component_references;
mod project_component_view;

pub use component_references::{referenced_task_labels, ComponentReferences, COMPONENT_REFERENCE_MAX};
pub use project_component_view::ProjectComponentView;
