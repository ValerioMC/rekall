use rekall_common::{Id, RekallError};
use rekall_model::token_usage::{ActiveModel, Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter};

pub async fn list_all(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().all(db).await?)
}

pub async fn list_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).all(db).await?)
}

pub async fn list_of_session(db: &impl ConnectionTrait, session_id: &str) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::SessionId.eq(session_id)).all(db).await?)
}

/// What a session spent, in place of whatever an earlier read of its transcript recorded.
pub async fn replace_of_session(db: &impl ConnectionTrait, session_id: &str, rows: Vec<Model>) -> Result<(), RekallError> {
    Entity::delete_many().filter(Column::SessionId.eq(session_id)).exec(db).await?;
    if rows.is_empty() {
        return Ok(());
    }
    Entity::insert_many(rows.into_iter().map(IntoActiveModel::<ActiveModel>::into_active_model)).exec(db).await?;
    Ok(())
}
