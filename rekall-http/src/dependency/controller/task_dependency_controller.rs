use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use rekall_console::dependency::TaskDependencyCommand;

use crate::dependency::{TaskDependencyRequest, TaskDependencyResponse};
use crate::shared::error::ApiResult;
use crate::shared::extract::{present, uuid, JsonBody};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/tasks/{id}/dependencies", get(list).post(add))
        .route("/api/tasks/{id}/dependencies/{depends_on_id}", delete(remove))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<TaskDependencyResponse>>> {
    Ok(Json(responses(state.task_dependencies.list_of(uuid("id", &id)?).await?)))
}

async fn add(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<TaskDependencyRequest>) -> ApiResult<Json<Vec<TaskDependencyResponse>>> {
    let task_id = uuid("id", &id)?;
    let raw = present("dependsOnTaskId", request.depends_on_task_id)?;
    let command = TaskDependencyCommand { depends_on_task_id: uuid("dependsOnTaskId", &raw)? };
    Ok(Json(responses(state.task_dependencies.add(task_id, command).await?)))
}

async fn remove(State(state): State<ApiState>, Path((id, depends_on_id)): Path<(String, String)>) -> ApiResult<Json<Vec<TaskDependencyResponse>>> {
    let views = state.task_dependencies.remove(uuid("id", &id)?, uuid("dependsOnId", &depends_on_id)?).await?;
    Ok(Json(responses(views)))
}

fn responses(views: Vec<rekall_console::dependency::TaskDependencyView>) -> Vec<TaskDependencyResponse> {
    views.into_iter().map(TaskDependencyResponse::from).collect()
}
