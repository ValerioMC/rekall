use rekall_common::{Id, RekallError};
use crate::project_section::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// A project's sections in the order the console shows them.
pub async fn list_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::ProjectId.eq(project_id))
        .order_by_asc(Column::Position)
        .order_by_asc(Column::CreatedAt)
        .all(db)
        .await?)
}
