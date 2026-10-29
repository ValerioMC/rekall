use axum::Router;

use super::{controller::installer_controller, controller::resource_controller, controller::skill_group_controller, controller::usage_controller};
use crate::ApiState;

pub(crate) fn routes() -> Router<ApiState> {
    Router::new()
        .merge(usage_controller::routes())
        .merge(resource_controller::routes())
        .merge(skill_group_controller::routes())
        .merge(installer_controller::routes())
}
