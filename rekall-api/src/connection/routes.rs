use axum::Router;

use super::{connection_controller, connection_credential_controller, jira_issue_controller, project_connection_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(connection_controller::routes())
        .merge(connection_credential_controller::routes())
        .merge(project_connection_controller::routes())
        .merge(jira_issue_controller::routes())
}
