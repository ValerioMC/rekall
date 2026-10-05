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

/// The company with this name, ignoring case. Names are free text, folded in Rust across all of
/// Unicode; the table holds tens of rows.
pub async fn find_by_name(db: &impl ConnectionTrait, name: &str) -> Result<Option<Model>, RekallError> {
    let all = Entity::find().all(db).await?;
    at_most_one(all.into_iter().filter(|c| equals_ignore_case(&c.name, name)).collect())
}

pub async fn list_by_name(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::Name).all(db).await?)
}

/// Whether a company other than `except` already has this name, ignoring case: two such names
/// would make a `company:` anchor ambiguous.
pub async fn name_taken(db: &impl ConnectionTrait, name: &str, except: Option<Id>) -> Result<bool, RekallError> {
    Ok(Entity::find()
        .all(db)
        .await?
        .iter()
        .any(|company| Some(company.id) != except && equals_ignore_case(&company.name, name)))
}
