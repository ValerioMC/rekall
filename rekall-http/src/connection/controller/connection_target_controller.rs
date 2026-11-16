use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::connection::{TargetSuggestionQuery, TargetSuggestionResponse};
use crate::shared::error::ApiResult;
use crate::shared::extract::uuid;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/connections/{id}/target-suggestions", get(suggestions))
}

async fn suggestions(State(state): State<ApiState>, Path(id): Path<String>, Query(query): Query<TargetSuggestionQuery>) -> ApiResult<Json<Vec<TargetSuggestionResponse>>> {
    Ok(Json(state.connection_targets.suggestions(uuid("id", &id)?, query.query.as_deref()).await?.into_iter().map(Into::into).collect()))
}
