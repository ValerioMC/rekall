use rekall_kernel::{Id, RekallError};
use crate::done_check::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// A lookup that must exist: the check, or a 404.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Check", id))
}

/// A project's checks, the ones for the whole project first (SQLite sorts an absent step first),
/// then each step's, every scope in its own order.
pub async fn list_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::ProjectId.eq(project_id))
        .order_by_asc(Column::StepId)
        .order_by_asc(Column::Position)
        .order_by_asc(Column::CreatedAt)
        .all(db)
        .await?)
}
