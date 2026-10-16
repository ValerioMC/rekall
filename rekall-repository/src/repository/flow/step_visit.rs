use rekall_common::{Id, RekallError};
use rekall_model::step_visit::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

/// A task's visits in the order the flow made them.
pub async fn list_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).order_by_asc(Column::Ordinal).all(db).await?)
}

pub async fn list_all(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::TaskId).order_by_asc(Column::Ordinal).all(db).await?)
}

/// Where the flow stands: the visit not yet left. The service keeps at most one per task.
pub async fn find_open(db: &impl ConnectionTrait, task_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .filter(Column::LeftAt.is_null())
        .order_by_desc(Column::Ordinal)
        .one(db)
        .await?)
}

/// The highest move number the task has used, zero before its first.
pub async fn last_ordinal(db: &impl ConnectionTrait, task_id: Id) -> Result<i32, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .order_by_desc(Column::Ordinal)
        .one(db)
        .await?
        .map_or(0, |visit| visit.ordinal))
}

pub async fn delete_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<u64, RekallError> {
    Ok(Entity::delete_many().filter(Column::TaskId.eq(task_id)).exec(db).await?.rows_affected)
}
