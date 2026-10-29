use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use rekall_web::error::ApiResult;
use rekall_web::extract::{uuid, JsonBody, Validator};
use rekall_web::created;

use crate::connection::{ConnectionAccessQuery, ConnectionAccessResponse, ConnectionDetailResponse, ConnectionRemovedResponse, ConnectionRequest, ConnectionResponse};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/companies/{id}/connections", get(list).post(create))
        .route("/api/connections/{id}", get(detail).put(update).delete(delete))
        .route("/api/connections/{id}/access", get(access))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<ConnectionResponse>>> {
    Ok(Json(state.connections.list(uuid("id", &id)?).await?.into_iter().map(ConnectionResponse::from).collect()))
}

async fn create(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ConnectionRequest>) -> ApiResult<Response> {
    let company_id = uuid("id", &id)?;
    Validator::new().not_blank("kind", request.kind.as_deref()).not_blank("title", request.title.as_deref()).finish()?;
    Ok(created(ConnectionResponse::from(state.connections.create(company_id, request.into()).await?)))
}

async fn detail(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<ConnectionDetailResponse>> {
    Ok(Json(state.connections.get(uuid("id", &id)?).await?.into()))
}

async fn update(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ConnectionRequest>) -> ApiResult<Json<ConnectionResponse>> {
    Ok(Json(state.connections.update(uuid("id", &id)?, request.into()).await?.into()))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<ConnectionRemovedResponse>> {
    Ok(Json(state.connections.delete(uuid("id", &id)?).await?.into()))
}

async fn access(State(state): State<ApiState>, Path(id): Path<String>, Query(query): Query<ConnectionAccessQuery>) -> ApiResult<Json<Vec<ConnectionAccessResponse>>> {
    Ok(Json(state.connection_access.page(uuid("id", &id)?, query.into()).await?.into_iter().map(ConnectionAccessResponse::from).collect()))
}
