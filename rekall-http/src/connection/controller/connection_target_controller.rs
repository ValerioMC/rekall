use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::connection::{GitlabChildrenQuery, GitlabNamespaceResponse, GitlabPathQuery, TargetCheckResponse, TargetSuggestionQuery, TargetSuggestionResponse};
use crate::shared::error::ApiResult;
use crate::shared::extract::uuid;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/connections/{id}/target-suggestions", get(suggestions))
        .route("/api/connections/{id}/target-check", get(check))
        .route("/api/connections/{id}/target-children", get(children))
}

async fn suggestions(State(state): State<ApiState>, Path(id): Path<String>, Query(query): Query<TargetSuggestionQuery>) -> ApiResult<Json<Vec<TargetSuggestionResponse>>> {
    Ok(Json(state.connection_targets.suggestions(uuid("id", &id)?, query.query.as_deref()).await?.into_iter().map(Into::into).collect()))
}

async fn check(State(state): State<ApiState>, Path(id): Path<String>, Query(query): Query<GitlabPathQuery>) -> ApiResult<Json<TargetCheckResponse>> {
    Ok(Json(state.gitlab_targets.check(uuid("id", &id)?, query.path.as_deref().unwrap_or_default()).await?.into()))
}

async fn children(State(state): State<ApiState>, Path(id): Path<String>, Query(query): Query<GitlabChildrenQuery>) -> ApiResult<Json<Vec<GitlabNamespaceResponse>>> {
    Ok(Json(state.gitlab_targets.children(uuid("id", &id)?, query.parent.as_deref(), query.query.as_deref()).await?.into_iter().map(Into::into).collect()))
}
