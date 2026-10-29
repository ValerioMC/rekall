use axum::extract::{Path, State};
use axum::routing::{post, put};
use axum::{Json, Router};
use rekall_web::error::ApiResult;
use rekall_web::extract::{uuid, JsonBody};

use crate::connection::{ConnectionCredentialRequest, ConnectionResponse, KubeContextResponse, KubeconfigPreviewRequest};
use crate::ApiState;

/// Credentials go in and never come out: every answer here is a connection or a context list.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/connections/{id}/credential", put(set).delete(clear))
        .route("/api/connections/kubeconfig/contexts", post(preview))
}

async fn set(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ConnectionCredentialRequest>) -> ApiResult<Json<ConnectionResponse>> {
    Ok(Json(state.connection_credentials.set(uuid("id", &id)?, request.into()).await?.into()))
}

async fn clear(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<ConnectionResponse>> {
    Ok(Json(state.connection_credentials.clear(uuid("id", &id)?).await?.into()))
}

async fn preview(State(state): State<ApiState>, JsonBody(request): JsonBody<KubeconfigPreviewRequest>) -> ApiResult<Json<Vec<KubeContextResponse>>> {
    Ok(Json(state.kubeconfig_preview.contexts(request.kubeconfig).await?.into_iter().map(KubeContextResponse::from).collect()))
}
