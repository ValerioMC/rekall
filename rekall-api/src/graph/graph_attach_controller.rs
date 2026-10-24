use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use rekall_service::step::TaskStepView;
use rekall_web::error::ApiResult;
use rekall_web::extract::{present, uuid, JsonBody};

use crate::graph::GraphAttachRequest;
use crate::ApiState;

/// Lays a deployment graph on a task; the answer is the task's whole checklist.
pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/tasks/{id}/graph", post(attach))
}

async fn attach(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<GraphAttachRequest>) -> ApiResult<Json<Vec<TaskStepView>>> {
    let task_id = uuid("id", &id)?;
    let graph_id = present("graphId", request.graph_id)?;
    Ok(Json(state.graph_attach.attach(task_id, graph_id).await?))
}
