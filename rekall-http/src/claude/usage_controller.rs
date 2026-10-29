use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use rekall_core::claude::ClaudeUsageView;
use rekall_kernel::RekallError;
use crate::shared::ApiError;

use super::usage_query::UsageQuery;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/claude/usage", get(usage))
}

/// Always 200; whether the figures are real is carried in `status`. `?refresh=true` is the
/// meter's "check again".
async fn usage(State(state): State<ApiState>, Query(query): Query<UsageQuery>) -> Result<Json<ClaudeUsageView>, ApiError> {
    let refresh = match query.refresh.as_deref() {
        None => false,
        Some(value) => parse_boolean(value).ok_or_else(|| {
            ApiError::Domain(RekallError::illegal(format!("Parameter 'refresh' is not a boolean: {value}")))
        })?,
    };
    Ok(Json(state.usage_meter.reading(refresh).await))
}

/// Lenient on purpose: an empty value is false, and `on`/`yes`/`1` read as true.
fn parse_boolean(value: &str) -> Option<bool> {
    match value.trim().to_lowercase().as_str() {
        "" => Some(false),
        "true" | "on" | "yes" | "1" => Some(true),
        "false" | "off" | "no" | "0" => Some(false),
        _ => None,
    }
}
