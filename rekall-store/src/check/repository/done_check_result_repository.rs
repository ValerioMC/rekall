use rekall_kernel::{Id, RekallError};
use crate::done_check_result::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

/// Every result kept for a task, steps and the task itself alike, the latest run first.
pub async fn list_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).order_by_desc(Column::RanAt).all(db).await?)
}
