use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use rekall_common::{jstr, RekallError};
use rekall_service::diagram::{DiagramDraft, DiagramService, DiagramSummaryView, DiagramTraceView, DiagramView};

use super::{created, no_content};
use crate::dto::{DiagramRequest, DiagramTraceQuery, SourceExcerptQuery, SourceExcerptResponse};
use crate::error::ApiResult;
use crate::extract::{optional_uuid, required, uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/diagrams", get(list).post(create))
        .route("/api/diagrams/{id}", get(get_one).put(replace).delete(delete))
        .route("/api/diagrams/{id}/source", get(source))
        .route("/api/projects/{id}/diagram-trace", get(trace))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<DiagramSummaryView>>> {
    Ok(Json(state.services.diagrams.list().await?))
}

async fn get_one(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<DiagramView>> {
    Ok(Json(state.services.diagrams.get(uuid("id", &id)?).await?))
}

async fn create(State(state): State<ApiState>, JsonBody(request): JsonBody<DiagramRequest>) -> ApiResult<Response> {
    let draft = draft_of(None, request)?;
    Ok(created(state.services.diagrams.write(draft).await?))
}

async fn replace(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<DiagramRequest>) -> ApiResult<Json<DiagramView>> {
    let draft = draft_of(Some(uuid("id", &id)?), request)?;
    Ok(Json(state.services.diagrams.write(draft).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.diagrams.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}

async fn source(State(state): State<ApiState>, Path(id): Path<String>, Query(query): Query<SourceExcerptQuery>) -> ApiResult<Json<SourceExcerptResponse>> {
    let file = required(query.file.filter(|file| !jstr::is_blank(file)))?;
    Ok(Json(state.sources.excerpt(uuid("id", &id)?, &file, query.start, query.end).await?))
}

async fn trace(State(state): State<ApiState>, Path(id): Path<String>, Query(query): Query<DiagramTraceQuery>) -> ApiResult<Json<Vec<DiagramTraceView>>> {
    let file = required(query.file.filter(|file| !jstr::is_blank(file)))?;
    let line = required(query.line)?;
    Ok(Json(state.services.diagrams.trace(uuid("id", &id)?, &file, line).await?))
}

fn draft_of(id: Option<rekall_common::Id>, request: DiagramRequest) -> ApiResult<DiagramDraft> {
    Validator::new()
        .not_blank("projectId", request.project_id.as_deref())
        .not_blank("title", request.title.as_deref())
        .finish()?;
    let graph = request.graph.ok_or_else(|| RekallError::illegal("A diagram needs a graph."))?;
    Ok(DiagramDraft {
        id,
        project_id: uuid("projectId", request.project_id.as_deref().unwrap_or_default())?,
        task_id: optional_uuid("taskId", request.task_id.as_deref())?,
        title: request.title.unwrap_or_default(),
        question: request.question.unwrap_or_default(),
        graph: DiagramService::read_graph(graph)?,
    })
}
