//! A project's components, as a context and the console read them. A context carries each one's
//! summary; the structured description is read on demand. `ProjectComponentService` takes a
//! session's write, which adds a component or rewrites the one with that name or folder, and
//! answers its read. Everything else a person does to them is the console's, in rekall-console.

mod event;
mod model;
pub mod rule;
mod service;
mod view;

pub use event::component_stream_event::ComponentStreamEvent;
pub use model::component_references::{names_task, referenced_task_labels, ComponentReferences, COMPONENT_REFERENCE_MAX};
pub use model::component_written::ComponentWritten;
pub use model::components_read::ComponentsRead;
pub use service::project_component_service::{ComponentDraft, ProjectComponentService};
pub use view::project_component_view::ProjectComponentView;
