use rekall_common::{Id, RekallError};
use rekall_model::commit_reference::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn find_by_task_id_and_commit_hash(db: &impl ConnectionTrait, task_id: Id, hash: &str) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .filter(Column::CommitHash.eq(hash))
        .all(db)
        .await?)
}

pub async fn find_all_by_order_by_created_at_desc(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_desc(Column::CreatedAt).all(db).await?)
}

/// The rows a task hands to `rekall_context`, in the order they were logged.
pub async fn find_by_task_id_and_in_context_true_order_by_created_at_asc(
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
