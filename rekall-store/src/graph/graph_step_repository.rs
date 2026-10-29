use rekall_common::{Id, RekallError};
use crate::graph_step::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

/// A graph's steps in order, the one it begins at first.
pub async fn list_of_graph(db: &impl ConnectionTrait, graph_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::GraphId.eq(graph_id)).order_by_asc(Column::Position).all(db).await?)
}

/// Deletes every step of the graph, and with them its routes.
pub async fn delete_of_graph(db: &impl ConnectionTrait, graph_id: Id) -> Result<(), RekallError> {
    Entity::delete_many().filter(Column::GraphId.eq(graph_id)).exec(db).await?;
    Ok(())
}
