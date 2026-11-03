use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use rekall_console::guard::GuardContextCommand;

use crate::guard::{GuardContextQuery, GuardContextResponse};
use crate::shared::error::ApiResult;
use crate::shared::extract::Validator;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/guard/context", get(context))
}

async fn context(State(state): State<ApiState>, Query(query): Query<GuardContextQuery>) -> ApiResult<Json<GuardContextResponse>> {
    Validator::new().not_blank("folder", query.folder.as_deref()).finish()?;
    let command = GuardContextCommand { folder: query.folder.unwrap_or_default() };
    Ok(Json(state.guard_context.context_of(command).await?.into()))
}
