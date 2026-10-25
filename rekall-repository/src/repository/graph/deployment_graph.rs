use rekall_common::{Id, RekallError};
use rekall_model::deployment_graph::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

/// A lookup that must exist: the graph, or a 404.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    Entity::find_by_id(id).one(db).await?.ok_or_else(|| RekallError::not_found("Deployment graph", id))
}

/// A project's graphs, oldest first.
pub async fn list_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::ProjectId.eq(project_id))
        .order_by_asc(Column::CreatedAt)
        .order_by_asc(Column::Title)
        .all(db)
        .await?)
}

/// The graph the project releases every task it runs with, if it chose one.
pub async fn release_default_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find().filter(Column::ProjectId.eq(project_id)).filter(Column::ReleaseDefault.eq(true)).one(db).await?)
}

pub async fn delete(db: &impl ConnectionTrait, id: Id) -> Result<(), RekallError> {
    Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}
