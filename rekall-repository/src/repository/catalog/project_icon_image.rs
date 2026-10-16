use rekall_common::{Id, Instant, RekallError};
use rekall_model::project_icon_image::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QuerySelect};

pub async fn find_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(project_id).one(db).await?)
}

/// The image's version alone, so a project's response does not read the bytes.
pub async fn find_version(db: &impl ConnectionTrait, project_id: Id) -> Result<Option<Instant>, RekallError> {
    Ok(Entity::find_by_id(project_id).select_only().column(Column::UpdatedAt).into_tuple().one(db).await?)
}

/// Which projects have an image and its version, without reading the bytes.
pub async fn find_all_versions(db: &impl ConnectionTrait) -> Result<Vec<(Id, Instant)>, RekallError> {
    Ok(Entity::find().select_only().column(Column::ProjectId).column(Column::UpdatedAt).into_tuple().all(db).await?)
}

pub async fn delete_by_project_id(db: &impl ConnectionTrait, project_id: Id) -> Result<(), RekallError> {
    Entity::delete_many().filter(Column::ProjectId.eq(project_id)).exec(db).await?;
    Ok(())
}
