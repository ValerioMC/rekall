use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, OptionalJsonBody};

use crate::catalog::TaskResponse;
use crate::connection::{JiraImportRequest, JiraIssueResponse};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/connections/{binding_id}/jira-issues", get(list))
        .route("/api/projects/{id}/connections/{binding_id}/jira-issues/import", post(import))
}

async fn list(State(state): State<ApiState>, Path((id, binding_id)): Path<(String, String)>) -> ApiResult<Json<Vec<JiraIssueResponse>>> {
    Ok(Json(state.jira_issues.list(uuid("id", &id)?, uuid("bindingId", &binding_id)?).await?.into_iter().map(Into::into).collect()))
}

async fn import(
    State(state): State<ApiState>,
    Path((id, binding_id)): Path<(String, String)>,
    OptionalJsonBody(request): OptionalJsonBody<JiraImportRequest>,
) -> ApiResult<Json<Vec<TaskResponse>>> {
    let request = request.unwrap_or_default();
    Ok(Json(state.jira_issues.import(uuid("id", &id)?, uuid("bindingId", &binding_id)?, request.into()).await?.into_iter().map(Into::into).collect()))
}
