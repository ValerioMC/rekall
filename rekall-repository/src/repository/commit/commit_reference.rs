use rekall_common::{Id, RekallError};
use rekall_model::commit_reference::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn find_on_task(db: &impl ConnectionTrait, task_id: Id, hash: &str) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .filter(Column::CommitHash.eq(hash))
        .all(db)
        .await?)
}

pub async fn list_newest_first(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_desc(Column::CreatedAt).all(db).await?)
}

pub async fn list_of_tasks(db: &impl ConnectionTrait, task_ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    if task_ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(Entity::find().filter(Column::TaskId.is_in(task_ids)).order_by_desc(Column::CreatedAt).all(db).await?)
}

/// The rows a task hands to `rekall_context`, in the order they were logged.
pub async fn list_in_context_of_task(
    db: &impl ConnectionTrait,
    task_id: Id,
) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .filter(Column::InContext.eq(true))
        .order_by_asc(Column::CreatedAt)
        .all(db)
        .await?)
}
