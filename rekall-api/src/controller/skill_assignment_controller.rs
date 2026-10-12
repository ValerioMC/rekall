use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};

use crate::dto::{SkillAssignmentRequest, SkillAssignmentsResponse, TagSkillResponse, TaskSkillResponse};
use crate::error::ApiResult;
use crate::extract::{uuid, JsonBody};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/skill-assignments", get(list))
        .route("/api/tasks/{id}/skills", put(set_for_task))
        .route("/api/steps/{id}/skills", put(set_for_step))
        .route("/api/tags/{id}/skills", put(set_for_tag))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<SkillAssignmentsResponse>> {
    Ok(Json(state.skill_assignments.list().await?))
}

async fn set_for_task(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<SkillAssignmentRequest>) -> ApiResult<Json<Vec<TaskSkillResponse>>> {
    Ok(Json(state.skill_assignments.set_for_task(uuid("id", &id)?, request.skills).await?))
}

async fn set_for_step(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<SkillAssignmentRequest>) -> ApiResult<Json<Vec<TaskSkillResponse>>> {
    Ok(Json(state.skill_assignments.set_for_step(uuid("id", &id)?, request.skills).await?))
}

async fn set_for_tag(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<SkillAssignmentRequest>) -> ApiResult<Json<Vec<TagSkillResponse>>> {
    Ok(Json(state.skill_assignments.set_for_tag(uuid("id", &id)?, request.skills).await?))
}
