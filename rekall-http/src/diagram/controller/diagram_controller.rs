use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use rekall_kernel::{jstr, RekallError};
use rekall_core::diagram::{DiagramDraft, DiagramService, DiagramSummaryView, DiagramView};
use serde_json::json;

use crate::shared::{created, no_content};
use crate::diagram::{DiagramRequest, SourceExcerptQuery, SourceExcerptResponse};
use crate::shared::error::ApiResult;
use crate::shared::extract::{optional_uuid, required, uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/diagrams", get(list).post(create))
        .route("/api/diagrams/{id}", get(get_one).put(replace).delete(delete))
        .route("/api/diagrams/{id}/source", get(source))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<DiagramSummaryView>>> {
    Ok(Json(state.diagrams.list().await?))
}

async fn get_one(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<DiagramView>> {
    Ok(Json(state.diagrams.get(uuid("id", &id)?).await?))
}

async fn create(State(state): State<ApiState>, JsonBody(request): JsonBody<DiagramRequest>) -> ApiResult<Response> {
    let draft = draft_of(None, request)?;
    Ok(created(state.diagrams.write(draft).await?))
}

async fn replace(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<DiagramRequest>) -> ApiResult<Json<DiagramView>> {
    let draft = draft_of(Some(uuid("id", &id)?), request)?;
    Ok(Json(state.diagrams.write(draft).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.diagram_catalog.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}

async fn source(State(state): State<ApiState>, Path(id): Path<String>, Query(query): Query<SourceExcerptQuery>) -> ApiResult<Json<SourceExcerptResponse>> {
    let file = required(query.file.filter(|file| !jstr::is_blank(file)))?;
    Ok(Json(state.sources.excerpt(uuid("id", &id)?, &file, query.start, query.end).await?.into()))
}

fn draft_of(id: Option<rekall_kernel::Id>, request: DiagramRequest) -> ApiResult<DiagramDraft> {
    Validator::new()
        .not_blank("projectId", request.project_id.as_deref())
        .not_blank("title", request.title.as_deref())
        .finish()?;
    let sheets = request.sheets.ok_or_else(|| RekallError::illegal("A diagram needs at least one sheet."))?;
    Ok(DiagramDraft {
        id,
        project_id: uuid("projectId", request.project_id.as_deref().unwrap_or_default())?,
        task_id: optional_uuid("taskId", request.task_id.as_deref())?,
        title: request.title.unwrap_or_default(),
        question: request.question.unwrap_or_default(),
        document: DiagramService::read_document(json!({ "sheets": sheets }))?,
    })
}
