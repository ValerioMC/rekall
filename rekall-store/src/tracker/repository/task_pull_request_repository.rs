use rekall_kernel::{Id, Instant, RekallError};
use crate::task_pull_request::{Column, Entity, Model};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter};

pub async fn find_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(task_id).one(db).await?)
}

pub async fn list_of_tasks(db: &impl ConnectionTrait, task_ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.is_in(task_ids)).all(db).await?)
}

/// Inserts the pull request of a task, or replaces what it holds.
pub async fn save(db: &impl ConnectionTrait, pull_request: Model) -> Result<(), RekallError> {
    pull_request.validate()?;
    Entity::insert(pull_request.into_active_model())
        .on_conflict(OnConflict::column(Column::TaskId).update_columns([Column::Number, Column::Url, Column::Failure, Column::MergedAt, Column::DeliveryVersion, Column::PublishedVersion, Column::UpdatedAt]).to_owned())
        .exec(db)
        .await?;
    Ok(())
}

pub async fn stage_delivery(db: &impl ConnectionTrait, task_id: Id, now: Instant) -> Result<(), RekallError> {
    let mut request = find_of_task(db, task_id).await?.unwrap_or_else(|| Model::empty(task_id, now));
    if request.merged_at.is_none() {
        request.delivery_version += 1;
        request.updated_at = now;
        save(db, request).await?;
    }
    Ok(())
}

/// Deletes the link to the task's pull request; the request itself stays on the task manager.
pub async fn delete_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<u64, RekallError> {
    Ok(Entity::delete_many().filter(Column::TaskId.eq(task_id)).exec(db).await?.rows_affected)
}

#[cfg(test)]
#[path = "../../../tests/unit/tracker/repository/task_pull_request_repository_tests.rs"]
mod tests;
