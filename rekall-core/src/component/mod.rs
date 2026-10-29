//! A project's components, as a context and the console read them. Only the console writes them
//! (rekall-console): a session reads which folders are parts of the project and never edits the list.

mod model;
mod view;

pub use model::component_references::{referenced_task_labels, ComponentReferences, COMPONENT_REFERENCE_MAX};
pub use view::project_component_view::ProjectComponentView;
