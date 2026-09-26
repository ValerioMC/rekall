use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use rekall_api::ApiError;
use rekall_common::RekallError;

use crate::ClaudeState;

use super::ClaudeUsageView;
use super::usage_query::UsageQuery;

pub fn routes() -> Router<ClaudeState> {
    Router::new().route("/api/claude/usage", get(usage))
}

/// Always 200; whether the figures are real is carried in `status`. `?refresh=true` is the
/// meter's "check again".
async fn usage(State(state): State<ClaudeState>, Query(query): Query<UsageQuery>) -> Result<Json<ClaudeUsageView>, ApiError> {
    let refresh = match query.refresh.as_deref() {
        None => false,
        Some(value) => parse_boolean(value).ok_or_else(|| {
            ApiError::Domain(RekallError::internal(
                "MethodArgumentTypeMismatchException",
                format!(
                    "Method parameter 'refresh': Failed to convert value of type 'java.lang.String' to required type \
                     'boolean'; Invalid boolean value [{value}]"
                ),
            ))
        })?,
    };
    Ok(Json(if refresh { state.usage.refresh().await } else { state.usage.current().await }))
}

/// Spring's `StringToBooleanConverter`.
fn parse_boolean(value: &str) -> Option<bool> {
    match value.trim().to_lowercase().as_str() {
        "" => Some(false),
        "true" | "on" | "yes" | "1" => Some(true),
        "false" | "off" | "no" | "0" => Some(false),
        _ => None,
    }
}
