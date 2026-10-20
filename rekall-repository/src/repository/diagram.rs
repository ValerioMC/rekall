use rekall_common::{Id, RekallError};
use rekall_model::diagram::{Column, Entity, Model};
use sea_orm::{ConnectionTrait, EntityTrait, QueryOrder, QuerySelect};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Diagram", id))
}

/// Every diagram without its document, newest first: what a list needs and no more.
pub async fn list_summaries_newest_first(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    let rows = Entity::find()
        .select_only()
        .columns([Column::Id, Column::ProjectId, Column::TaskId, Column::Title, Column::Question, Column::Kinds, Column::Walkthrough, Column::CreatedAt, Column::UpdatedAt])
        .expr_as(sea_orm::sea_query::Expr::val(""), "document_json")
        .order_by_desc(Column::UpdatedAt)
        .into_model::<Model>()
        .all(db)
        .await?;
    Ok(rows)
}
