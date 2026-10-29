use rekall_kernel::{Id, RekallError};
use crate::project_dependency::{ActiveModel, Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

/// A project's dependencies by ecosystem, then name.
pub async fn list_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::ProjectId.eq(project_id))
        .order_by_asc(Column::Ecosystem)
        .order_by_asc(Column::Name)
        .order_by_asc(Column::Version)
        .all(db)
        .await?)
}

/// Every dependency Rekall holds, for a scan that reads all projects at once.
pub async fn list_all(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .order_by_asc(Column::ProjectId)
        .order_by_asc(Column::Ecosystem)
        .order_by_asc(Column::Name)
        .order_by_asc(Column::Version)
        .all(db)
        .await?)
}

/// Replaces the project's whole list with `rows`.
pub async fn replace_of_project(db: &impl ConnectionTrait, project_id: Id, rows: Vec<Model>) -> Result<(), RekallError> {
    Entity::delete_many().filter(Column::ProjectId.eq(project_id)).exec(db).await?;
    if rows.is_empty() {
        return Ok(());
    }
    for row in &rows {
        row.validate()?;
    }
    let active: Vec<ActiveModel> = rows.into_iter().map(Into::into).collect();
    Entity::insert_many(active).exec(db).await?;
    Ok(())
}
