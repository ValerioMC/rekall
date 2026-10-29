use rekall_common::Id;
use rekall_model::{project, project_connection};

/// A project that binds a connection, and what it uses it for: once per binding, since a project
/// may bind one connection for several targets.
#[derive(Clone, Debug)]
pub struct ConnectionProjectView {
    pub project_id: Id,
    pub label: String,
    pub title: String,
    pub binding_label: String,
    pub target: Option<String>,
}

impl ConnectionProjectView {
    pub fn of(binding: &project_connection::Model, project: &project::Model) -> Self {
        Self {
            project_id: project.id,
            label: project.label.clone(),
            title: project.title.clone(),
            binding_label: binding.label.clone(),
            target: binding.target.clone(),
        }
    }
}
