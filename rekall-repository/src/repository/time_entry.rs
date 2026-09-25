use rekall_common::{Id, RekallError};
use rekall_model::time_entry::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

use super::at_most_one;

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn find_by_task_id_and_stopped_at_is_null(db: &impl ConnectionTrait, task_id: Id) -> Result<Option<Model>, RekallError> {
    at_most_one(
        Entity::find()
            .filter(Column::TaskId.eq(task_id))
            .filter(Column::StoppedAt.is_null())
            .all(db)
            .await?,
    )
}

pub async fn find_all_by_order_by_started_at_desc(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_desc(Column::StartedAt).all(db).await?)
}

pub async fn find_all_by_stopped_at_is_null(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::StoppedAt.is_null()).all(db).await?)
}
