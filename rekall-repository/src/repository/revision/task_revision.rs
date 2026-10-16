use rekall_common::{Id, RekallError};
use rekall_model::task_revision::{Column, Entity, Model};
use rekall_model::RevisionKind;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

pub async fn list_of_task_newest_first(
    db: &impl ConnectionTrait,
    task_id: Id,
    kind: RevisionKind,
) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .filter(Column::Kind.eq(kind))
        .order_by_desc(Column::CreatedAt)
        .all(db)
        .await?)
}

pub async fn find_newest_of_task(
    db: &impl ConnectionTrait,
    task_id: Id,
    kind: RevisionKind,
) -> Result<Option<Model>, RekallError> {
    Ok(list_of_task_newest_first(db, task_id, kind).await?.into_iter().next())
}

pub async fn find_on_task(db: &impl ConnectionTrait, id: Id, task_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::Id.eq(id))
        .filter(Column::TaskId.eq(task_id))
        .one(db)
        .await?)
}
