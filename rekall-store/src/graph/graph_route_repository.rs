use rekall_common::{Id, RekallError};
use crate::graph_route::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

/// A graph's routes, each step's in the order they were written.
pub async fn list_of_graph(db: &impl ConnectionTrait, graph_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::GraphId.eq(graph_id)).order_by_asc(Column::Position).all(db).await?)
}
