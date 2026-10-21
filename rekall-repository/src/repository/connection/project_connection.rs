use rekall_common::{Id, RekallError};
use rekall_model::project_connection::{Column, Entity, Model};
use rekall_model::{external_connection, project};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};

/// The project's binding to the connection, if it has one.
pub async fn find(db: &impl ConnectionTrait, project_id: Id, connection_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::ProjectId.eq(project_id))
        .filter(Column::ConnectionId.eq(connection_id))
        .one(db)
        .await?)
}

/// A binding that must exist: the project's use of the connection, or a 404.
pub async fn require(db: &impl ConnectionTrait, project_id: Id, connection_id: Id) -> Result<Model, RekallError> {
    find(db, project_id, connection_id)
        .await?
        .ok_or_else(|| RekallError::not_found("ProjectConnection", connection_id))
}

/// The project's bindings with the connection each names, by the connection's label.
pub async fn list_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<(Model, external_connection::Model)>, RekallError> {
    let rows = Entity::find()
        .find_also_related(external_connection::Entity)
        .filter(Column::ProjectId.eq(project_id))
        .order_by_asc(external_connection::Column::Label)
        .all(db)
        .await?;
    Ok(rows.into_iter().filter_map(|(binding, connection)| connection.map(|connection| (binding, connection))).collect())
}

/// The projects that bind the connection, by project label.
pub async fn list_of_connection(db: &impl ConnectionTrait, connection_id: Id) -> Result<Vec<(Model, project::Model)>, RekallError> {
    let rows = Entity::find()
        .find_also_related(project::Entity)
        .filter(Column::ConnectionId.eq(connection_id))
        .order_by_asc(project::Column::Label)
        .all(db)
        .await?;
    Ok(rows.into_iter().filter_map(|(binding, project)| project.map(|project| (binding, project))).collect())
}

pub async fn bind(db: &impl ConnectionTrait, binding: Model) -> Result<Model, RekallError> {
    binding.validate()?;
    Ok(binding.into_active_model().insert(db).await?)
}

/// Whether there was a binding to remove.
pub async fn unbind(db: &impl ConnectionTrait, project_id: Id, connection_id: Id) -> Result<bool, RekallError> {
    let deleted = Entity::delete_many()
        .filter(Column::ProjectId.eq(project_id))
        .filter(Column::ConnectionId.eq(connection_id))
        .exec(db)
        .await?;
    Ok(deleted.rows_affected > 0)
}
