use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::Deserialize;

use super::{created, no_content};
use crate::dto::{DocumentRequest, DocumentResponse};
use crate::error::ApiResult;
use crate::extract::{optional_uuid, required, uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/documents", get(list).post(create))
        .route("/api/documents/search", get(search))
        .route("/api/documents/{id}", put(update).delete(delete))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    task_id: Option<String>,
    project_id: Option<String>,
}

#[derive(Deserialize)]
struct SearchQuery {
    query: Option<String>,
}

fn validate(request: &DocumentRequest) -> ApiResult<()> {
    Validator::new()
        .not_blank("title", request.title.as_deref())
        .not_blank("kind", request.kind.as_deref())
        .finish()
}

async fn list(State(state): State<ApiState>, Query(query): Query<ListQuery>) -> ApiResult<Json<Vec<DocumentResponse>>> {
    let task_id = optional_uuid("taskId", query.task_id.as_deref())?;
    let project_id = optional_uuid("projectId", query.project_id.as_deref())?;
    Ok(Json(state.documents.list(task_id, project_id).await?))
}

async fn search(State(state): State<ApiState>, Query(query): Query<SearchQuery>) -> ApiResult<Json<Vec<DocumentResponse>>> {
    let term = required(query.query)?;
    Ok(Json(state.documents.search(Some(&term)).await?))
}

async fn create(State(state): State<ApiState>, JsonBody(request): JsonBody<DocumentRequest>) -> ApiResult<Response> {
    validate(&request)?;
    Ok(created(state.documents.create(request).await?))
}

async fn update(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<DocumentRequest>,
) -> ApiResult<Json<DocumentResponse>> {
    let id = uuid("id", &id)?;
    validate(&request)?;
    Ok(Json(state.documents.update(id, request).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.documents.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
