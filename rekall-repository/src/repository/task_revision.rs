use rekall_common::{Id, RekallError};
use rekall_model::task_revision::{Column, Entity, Model};
use rekall_model::RevisionKind;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

pub async fn find_by_task_id_and_kind_order_by_created_at_desc(
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

pub async fn find_first_by_task_id_and_kind_order_by_created_at_desc(
    db: &impl ConnectionTrait,
    task_id: Id,
    kind: RevisionKind,
) -> Result<Option<Model>, RekallError> {
    Ok(find_by_task_id_and_kind_order_by_created_at_desc(db, task_id, kind).await?.into_iter().next())
}

pub async fn find_by_id_and_task_id(db: &impl ConnectionTrait, id: Id, task_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::Id.eq(id))
        .filter(Column::TaskId.eq(task_id))
        .one(db)
        .await?)
}
