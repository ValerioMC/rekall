use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use rekall_service::token::{TokenDayView, TokenTallyView};

use rekall_web::error::ApiResult;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/token-usage", get(list)).route("/api/token-usage/daily", get(daily))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<TokenTallyView>>> {
    Ok(Json(state.token_usage.tallies().await?))
}

async fn daily(State(state): State<ApiState>) -> ApiResult<Json<Vec<TokenDayView>>> {
    Ok(Json(state.token_usage.days().await?))
}
