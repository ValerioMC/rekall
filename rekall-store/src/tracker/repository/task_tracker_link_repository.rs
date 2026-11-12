use rekall_kernel::{Id, RekallError};
use crate::task_tracker_link::{Column, Entity, Model};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter};

pub async fn find_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(task_id).one(db).await?)
}

pub async fn list_of_tasks(db: &impl ConnectionTrait, task_ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.is_in(task_ids)).all(db).await?)
}

/// Inserts the link of a task, or replaces what it holds.
pub async fn save(db: &impl ConnectionTrait, link: Model) -> Result<(), RekallError> {
    link.validate()?;
    Entity::insert(link.into_active_model())
        .on_conflict(
            OnConflict::column(Column::TaskId)
                .update_columns([Column::IssueKey, Column::IssueUrl, Column::SyncError, Column::UpdatedAt])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}
