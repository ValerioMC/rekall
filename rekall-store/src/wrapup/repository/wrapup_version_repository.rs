use rekall_kernel::{Id, RekallError};
use crate::wrapup::wrapup_version::{Column, Entity, Model, KEPT_PER_TASK};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};

pub async fn list_of_task_newest_first(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).order_by_desc(Column::Number).all(db).await?)
}

/// A version of the task, so that an id from another task's list reads as absent.
pub async fn find_of_task(db: &impl ConnectionTrait, task_id: Id, version_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).filter(Column::Id.eq(version_id)).one(db).await?)
}

/// Stores the next version of the task and drops the ones past `KEPT_PER_TASK`.
pub async fn append(db: &impl ConnectionTrait, version: Model) -> Result<Model, RekallError> {
    let numbered = Model { number: next_number(db, version.task_id).await?, ..version };
    numbered.clone().into_active_model().insert(db).await?;
    trim(db, numbered.task_id).await?;
    Ok(numbered)
}

async fn next_number(db: &impl ConnectionTrait, task_id: Id) -> Result<i32, RekallError> {
    let latest = Entity::find().filter(Column::TaskId.eq(task_id)).order_by_desc(Column::Number).one(db).await?;
    Ok(latest.map_or(1, |version| version.number + 1))
}

async fn trim(db: &impl ConnectionTrait, task_id: Id) -> Result<(), RekallError> {
    let kept_and_surplus = list_of_task_newest_first(db, task_id).await?;
    for version in kept_and_surplus.into_iter().skip(KEPT_PER_TASK) {
        Entity::delete_by_id(version.id).exec(db).await?;
    }
    Ok(())
}
