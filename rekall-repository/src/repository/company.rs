use rekall_common::{Id, RekallError};
use rekall_model::company::{Column, Entity, Model};
use sea_orm::{ConnectionTrait, EntityTrait, QueryOrder};

use super::{at_most_one, equals_ignore_case};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// The company by id, or `NotFound` when there is none.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Company", id))
}

pub async fn find_by_name_ignore_case(db: &impl ConnectionTrait, name: &str) -> Result<Option<Model>, RekallError> {
    let all = Entity::find().all(db).await?;
    at_most_one(all.into_iter().filter(|c| equals_ignore_case(&c.name, name)).collect())
}

pub async fn find_all_by_order_by_name_asc(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::Name).all(db).await?)
}
