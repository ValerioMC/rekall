use rekall_model::{external_connection, project, project_connection};

use super::{ConnectionProjectView, ConnectionView};

/// One connection with the projects that bind it, a row per binding.
#[derive(Clone, Debug)]
pub struct ConnectionDetailView {
    pub connection: ConnectionView,
    pub projects: Vec<ConnectionProjectView>,
}

impl ConnectionDetailView {
    pub fn of(connection: &external_connection::Model, projects: &[(project_connection::Model, project::Model)], project_count: usize) -> Self {
        Self {
            connection: ConnectionView::of(connection, project_count),
            projects: projects.iter().map(|(binding, project)| ConnectionProjectView::of(binding, project)).collect(),
        }
    }
}
