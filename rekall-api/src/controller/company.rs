use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};

use super::{created, no_content};
use crate::dto::{CompanyRequest, CompanyResponse};
use crate::error::ApiResult;
use crate::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/companies", get(list).post(create))
        .route("/api/companies/{id}", put(update).delete(delete))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<CompanyResponse>>> {
    Ok(Json(state.catalog.list_companies().await?))
}

async fn create(State(state): State<ApiState>, JsonBody(request): JsonBody<CompanyRequest>) -> ApiResult<Response> {
    Validator::new().not_blank("name", request.name.as_deref()).finish()?;
    Ok(created(state.catalog.create_company(request).await?))
}

async fn update(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<CompanyRequest>,
) -> ApiResult<Json<CompanyResponse>> {
    let id = uuid("id", &id)?;
    Validator::new().not_blank("name", request.name.as_deref()).finish()?;
    Ok(Json(state.catalog.update_company(id, request).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.catalog.delete_company(uuid("id", &id)?).await?;
    Ok(no_content())
}
