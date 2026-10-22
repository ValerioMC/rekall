use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_web::error::ApiResult;
use rekall_web::extract::{uuid, OptionalJsonBody};

use crate::catalog::TaskResponse;
use crate::connection::{JiraImportRequest, JiraIssueResponse};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/connections/{connection_id}/jira-issues", get(list))
        .route("/api/projects/{id}/connections/{connection_id}/jira-issues/import", post(import))
}

async fn list(State(state): State<ApiState>, Path((id, connection_id)): Path<(String, String)>) -> ApiResult<Json<Vec<JiraIssueResponse>>> {
    Ok(Json(state.jira_issues.list(uuid("id", &id)?, uuid("connectionId", &connection_id)?).await?))
}

async fn import(
    State(state): State<ApiState>,
    Path((id, connection_id)): Path<(String, String)>,
    OptionalJsonBody(request): OptionalJsonBody<JiraImportRequest>,
) -> ApiResult<Json<Vec<TaskResponse>>> {
    let request = request.unwrap_or_default();
    Ok(Json(state.jira_issues.import(uuid("id", &id)?, uuid("connectionId", &connection_id)?, request).await?))
}
