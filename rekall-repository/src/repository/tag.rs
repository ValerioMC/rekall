use rekall_common::{Id, RekallError};
use rekall_model::tag::{Column, Entity, Model};
use sea_orm::{ConnectionTrait, EntityTrait, QueryOrder};

use super::equals_ignore_case;

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn find_all_by_order_by_name_asc(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::Name).all(db).await?)
}

pub async fn exists_by_name_ignore_case(db: &impl ConnectionTrait, name: &str) -> Result<bool, RekallError> {
    Ok(Entity::find().all(db).await?.iter().any(|t| equals_ignore_case(&t.name, name)))
}

pub async fn exists_by_name_ignore_case_and_id_not(db: &impl ConnectionTrait, name: &str, id: Id) -> Result<bool, RekallError> {
    Ok(Entity::find()
        .all(db)
        .await?
        .iter()
        .any(|t| t.id != id && equals_ignore_case(&t.name, name)))
}
