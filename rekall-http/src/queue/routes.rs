use axum::Router;

use super::queue_controller;
use crate::ApiState;

pub(crate) fn routes() -> Router<ApiState> {
    queue_controller::routes()
}
