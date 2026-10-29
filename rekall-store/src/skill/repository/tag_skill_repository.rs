use rekall_kernel::{Id, RekallError};
use crate::tag_skill::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};

pub async fn list_all(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::TagId).order_by_asc(Column::Position).all(db).await?)
}

pub async fn list_of_tag(db: &impl ConnectionTrait, tag_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TagId.eq(tag_id)).order_by_asc(Column::Position).all(db).await?)
}

/// Replaces the tag's skills with `skills`, in order.
pub async fn replace(db: &impl ConnectionTrait, tag_id: Id, skills: Vec<Model>) -> Result<(), RekallError> {
    Entity::delete_many().filter(Column::TagId.eq(tag_id)).exec(db).await?;
    for skill in skills {
        Entity::insert(skill.into_active_model()).exec(db).await?;
    }
    Ok(())
}
