use rekall_common::RekallError;
use rekall_model::skill_placement::{ActiveModel, Column, Entity, Model};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};

pub async fn list_all(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::GroupLabel).order_by_asc(Column::SkillPath).all(db).await?)
}

/// Puts the skill under the group, moving it out of the one it was in.
pub async fn place(db: &impl ConnectionTrait, placement: Model) -> Result<(), RekallError> {
    let row: ActiveModel = placement.into_active_model();
    Entity::insert(row)
        .on_conflict(OnConflict::column(Column::SkillPath).update_columns([Column::GroupLabel, Column::PlacedAt]).to_owned())
        .exec(db)
        .await?;
    Ok(())
}

pub async fn remove(db: &impl ConnectionTrait, skill_path: &str) -> Result<(), RekallError> {
    Entity::delete_by_id(skill_path.to_string()).exec(db).await?;
    Ok(())
}

/// Moves every skill of one group to another label; the number moved.
pub async fn rename_group(db: &impl ConnectionTrait, from: &str, to: &str) -> Result<u64, RekallError> {
    let moved = Entity::update_many()
        .col_expr(Column::GroupLabel, sea_orm::sea_query::Expr::value(to.to_string()))
        .filter(Column::GroupLabel.eq(from))
        .exec(db)
        .await?;
    Ok(moved.rows_affected)
}
