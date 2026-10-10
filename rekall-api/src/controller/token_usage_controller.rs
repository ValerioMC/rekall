use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use rekall_service::token::TokenTallyView;

use crate::error::ApiResult;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/token-usage", get(list))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<TokenTallyView>>> {
    Ok(Json(state.token_usage.tallies().await?))
}
