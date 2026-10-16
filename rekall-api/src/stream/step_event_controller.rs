use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/steps/stream", get(stream))
}

async fn stream(State(state): State<ApiState>) -> impl IntoResponse {
    state.stream.open()
}
