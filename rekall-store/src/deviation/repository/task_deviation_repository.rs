use rekall_kernel::{Id, RekallError};
use crate::task_deviation::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

/// A task's deviations, by the step they refer to.
pub async fn list_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).order_by_asc(Column::StepNumber).all(db).await?)
}

pub async fn find_of_step(db: &impl ConnectionTrait, task_id: Id, step_number: i32) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).filter(Column::StepNumber.eq(step_number)).one(db).await?)
}
