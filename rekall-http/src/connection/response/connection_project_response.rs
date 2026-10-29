use rekall_kernel::Id;
use rekall_console::connection::ConnectionProjectView;
use serde::Serialize;

/// A project that binds a connection, and what it uses it for: once per binding, since a project
/// may bind one connection for several targets.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProjectResponse {
    pub project_id: Id,
    pub label: String,
    pub title: String,
    pub binding_label: String,
    pub target: Option<String>,
}

impl From<ConnectionProjectView> for ConnectionProjectResponse {
    fn from(view: ConnectionProjectView) -> Self {
        Self { project_id: view.project_id, label: view.label, title: view.title, binding_label: view.binding_label, target: view.target }
    }
}
