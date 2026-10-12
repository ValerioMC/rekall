use axum::extract::State;
use axum::routing::{get, put};
use axum::{Json, Router};
use rekall_api::error::ApiResult;
use rekall_api::extract::{present, JsonBody};

use crate::ClaudeState;

use super::skill_group_requests::{PlaceSkillRequest, RenameGroupRequest};
use super::SkillPlacementView;

pub fn routes() -> Router<ClaudeState> {
    Router::new()
        .route("/api/claude/resources/skill-groups", get(placements))
        .route("/api/claude/resources/skill-groups/placement", put(place))
        .route("/api/claude/resources/skill-groups/rename", put(rename))
}

async fn placements(State(state): State<ClaudeState>) -> ApiResult<Json<Vec<SkillPlacementView>>> {
    Ok(Json(state.skill_groups.placements().await?))
}

async fn place(State(state): State<ClaudeState>, JsonBody(request): JsonBody<PlaceSkillRequest>) -> ApiResult<Json<Vec<SkillPlacementView>>> {
    let skill_path = present("skillPath", request.skill_path)?;
    Ok(Json(state.skill_groups.place(&skill_path, request.group.as_deref()).await?))
}

async fn rename(State(state): State<ClaudeState>, JsonBody(request): JsonBody<RenameGroupRequest>) -> ApiResult<Json<Vec<SkillPlacementView>>> {
    let from = present("from", request.from)?;
    let to = present("to", request.to)?;
    Ok(Json(state.skill_groups.rename_group(&from, &to).await?))
}
