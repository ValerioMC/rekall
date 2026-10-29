use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use rekall_service::step::TaskStepView;
use rekall_web::error::ApiResult;
use rekall_web::extract::{present, uuid, JsonBody};

use crate::graph::{GraphAttachRequest, ReleaseOnRunResponse};
use crate::ApiState;

/// Lays a deployment graph on a task; the answer is the task's whole checklist.
pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/tasks/{id}/graph", post(attach)).route("/api/tasks/{id}/release", post(release_on_run))
}

/// Called by the console as it starts a session on the task: lays the project's release graph
/// unless the task has one or is left unreleased.
async fn release_on_run(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<ReleaseOnRunResponse>> {
    Ok(Json(state.graph_attach.release_on_run(uuid("id", &id)?).await?.into()))
}

async fn attach(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<GraphAttachRequest>) -> ApiResult<Json<Vec<TaskStepView>>> {
    let task_id = uuid("id", &id)?;
    let graph_id = present("graphId", request.graph_id)?;
    Ok(Json(state.graph_attach.attach(task_id, graph_id).await?))
}
