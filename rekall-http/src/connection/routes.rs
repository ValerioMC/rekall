use axum::Router;

use super::{controller::connection_check_controller, controller::connection_controller, controller::connection_credential_controller, controller::jira_issue_controller, controller::project_connection_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(connection_controller::routes())
        .merge(connection_check_controller::routes())
        .merge(connection_credential_controller::routes())
        .merge(project_connection_controller::routes())
        .merge(jira_issue_controller::routes())
}
