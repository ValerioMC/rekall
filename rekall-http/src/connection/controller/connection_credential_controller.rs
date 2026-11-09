use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, JsonBody};

use crate::connection::{ConnectionCredentialRequest, ConnectionResponse, KubeContextResponse, KubeconfigPreviewRequest, RevealedCredentialResponse};
use crate::ApiState;

/// Credentials go in as a connection's write and come out only on the `GET`, one at a time, for the console's reveal and copy buttons.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/connections/{id}/credential", get(reveal).put(set).delete(clear))
        .route("/api/connections/kubeconfig/contexts", post(preview))
}

async fn reveal(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<RevealedCredentialResponse>> {
    Ok(Json(state.connection_credential_reveals.reveal(uuid("id", &id)?).await?.into()))
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
