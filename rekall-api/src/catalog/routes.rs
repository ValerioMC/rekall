use axum::Router;

use super::{company_controller, company_logo_controller, epic_controller, project_controller, project_icon_controller, task_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(company_controller::routes())
        .merge(company_logo_controller::routes())
        .merge(epic_controller::routes())
        .merge(project_controller::routes())
        .merge(project_icon_controller::routes())
        .merge(task_controller::routes())
}
