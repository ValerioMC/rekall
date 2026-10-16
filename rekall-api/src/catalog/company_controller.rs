use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};

use rekall_web::{created, no_content};
use crate::catalog::{CompanyRequest, CompanyResponse};
use rekall_web::error::ApiResult;
use rekall_web::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/companies", get(list).post(create))
        .route("/api/companies/{id}", put(update).delete(delete))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<CompanyResponse>>> {
    Ok(Json(state.companies.list().await?))
}

async fn create(State(state): State<ApiState>, JsonBody(request): JsonBody<CompanyRequest>) -> ApiResult<Response> {
    Validator::new().not_blank("name", request.name.as_deref()).finish()?;
    Ok(created(state.companies.create(request).await?))
}

async fn update(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<CompanyRequest>,
) -> ApiResult<Json<CompanyResponse>> {
    let id = uuid("id", &id)?;
    Validator::new().not_blank("name", request.name.as_deref()).finish()?;
    Ok(Json(state.companies.update(id, request).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.companies.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
