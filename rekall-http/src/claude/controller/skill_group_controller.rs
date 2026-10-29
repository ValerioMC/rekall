use axum::extract::State;
use axum::routing::{get, put};
use axum::{Json, Router};
use crate::shared::error::ApiResult;
use crate::shared::extract::{present, JsonBody};

use rekall_core::skill::SkillPlacementView;

use crate::claude::{PlaceSkillRequest, RenameGroupRequest};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/claude/resources/skill-groups", get(placements))
        .route("/api/claude/resources/skill-groups/placement", put(place))
        .route("/api/claude/resources/skill-groups/rename", put(rename))
}

async fn placements(State(state): State<ApiState>) -> ApiResult<Json<Vec<SkillPlacementView>>> {
    Ok(Json(state.skill_groups.placements().await?))
}

async fn place(State(state): State<ApiState>, JsonBody(request): JsonBody<PlaceSkillRequest>) -> ApiResult<Json<Vec<SkillPlacementView>>> {
    let skill_path = present("skillPath", request.skill_path)?;
    Ok(Json(state.skill_groups.place(&skill_path, request.group.as_deref()).await?))
}

async fn rename(State(state): State<ApiState>, JsonBody(request): JsonBody<RenameGroupRequest>) -> ApiResult<Json<Vec<SkillPlacementView>>> {
    let from = present("from", request.from)?;
    let to = present("to", request.to)?;
    Ok(Json(state.skill_groups.rename_group(&from, &to).await?))
}
