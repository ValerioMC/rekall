use rekall_common::Id;
use rekall_model::{project, project_connection};
use serde::Serialize;

/// A project that binds a connection, and what it uses it for.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProjectResponse {
    pub project_id: Id,
    pub label: String,
    pub title: String,
    pub target: Option<String>,
}

impl ConnectionProjectResponse {
    pub fn of(binding: &project_connection::Model, project: &project::Model) -> Self {
        Self { project_id: project.id, label: project.label.clone(), title: project.title.clone(), target: binding.target.clone() }
    }
}
