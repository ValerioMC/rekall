use rekall_store::{project, project_connection};

use crate::connection::{ConnectionCompanyView, ConnectionProjectView, ConnectionView};

/// One connection with the companies it is shared with and the projects that bind it, a row per binding.
#[derive(Clone, Debug)]
pub struct ConnectionDetailView {
    pub connection: ConnectionView,
    pub companies: Vec<ConnectionCompanyView>,
    pub projects: Vec<ConnectionProjectView>,
}

impl ConnectionDetailView {
    pub fn of(connection: ConnectionView, companies: Vec<ConnectionCompanyView>, projects: &[(project_connection::Model, project::Model)]) -> Self {
        Self { connection, companies, projects: projects.iter().map(|(binding, project)| ConnectionProjectView::of(binding, project)).collect() }
    }
}
