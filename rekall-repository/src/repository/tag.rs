use rekall_common::{Id, RekallError};
use rekall_model::tag::{Column, Entity, Model};
use sea_orm::{ConnectionTrait, EntityTrait, QueryOrder};

use super::equals_ignore_case;

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn list_by_name(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::Name).all(db).await?)
}

/// Whether a tag other than `except` already has this name, ignoring case. Names are free text,
/// folded in Rust across all of Unicode; the table holds tens of rows.
pub async fn name_taken(db: &impl ConnectionTrait, name: &str, except: Option<Id>) -> Result<bool, RekallError> {
    Ok(Entity::find()
        .all(db)
        .await?
        .iter()
        .any(|tag| Some(tag.id) != except && equals_ignore_case(&tag.name, name)))
}
