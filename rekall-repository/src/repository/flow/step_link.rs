use rekall_common::{Id, RekallError};
use rekall_model::step_link::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

/// The route by id, or `NotFound` when there is none.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    Entity::find_by_id(id).one(db).await?.ok_or_else(|| RekallError::not_found("Route", id))
}

pub async fn list_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .order_by_asc(Column::Position)
        .order_by_asc(Column::CreatedAt)
        .all(db)
        .await?)
}

pub async fn list_all(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .order_by_asc(Column::TaskId)
        .order_by_asc(Column::Position)
        .order_by_asc(Column::CreatedAt)
        .all(db)
        .await?)
}

/// The routes leaving one step, in the order the person ranked them.
pub async fn list_from(db: &impl ConnectionTrait, from_step_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::FromStepId.eq(from_step_id))
        .order_by_asc(Column::Position)
        .order_by_asc(Column::CreatedAt)
        .all(db)
        .await?)
}

pub async fn delete(db: &impl ConnectionTrait, id: Id) -> Result<(), RekallError> {
    Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}
