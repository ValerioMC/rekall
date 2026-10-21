use rekall_common::{Id, RekallError};
use rekall_model::connection_access::{Column, Entity, Model};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, QuerySelect};

pub async fn record(db: &impl ConnectionTrait, access: Model) -> Result<Model, RekallError> {
    access.validate()?;
    Ok(access.into_active_model().insert(db).await?)
}

/// One page of the connection's log, newest first.
pub async fn list_of_connection(db: &impl ConnectionTrait, connection_id: Id, offset: u64, limit: u64) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::ConnectionId.eq(connection_id))
        .order_by_desc(Column::AccessedAt)
        .order_by_desc(Column::Id)
        .offset(offset)
        .limit(limit)
        .all(db)
        .await?)
}
