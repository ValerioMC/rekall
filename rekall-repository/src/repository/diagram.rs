use rekall_common::{Id, RekallError};
use rekall_model::diagram::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// Every diagram without its graph, newest first: what a list needs and no more.
pub async fn find_all_without_graph_order_by_updated_at_desc(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    let rows = Entity::find()
        .select_only()
        .columns([Column::Id, Column::ProjectId, Column::TaskId, Column::Title, Column::Question, Column::NodeCount, Column::EdgeCount, Column::CreatedAt, Column::UpdatedAt])
        .expr_as(sea_orm::sea_query::Expr::val(""), "graph_json")
        .order_by_desc(Column::UpdatedAt)
        .into_model::<Model>()
        .all(db)
        .await?;
    Ok(rows)
}

pub async fn find_all_by_project_id(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::ProjectId.eq(project_id)).order_by_desc(Column::UpdatedAt).all(db).await?)
}
