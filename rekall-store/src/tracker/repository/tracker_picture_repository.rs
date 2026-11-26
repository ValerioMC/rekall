use rekall_kernel::{Id, RekallError};
use crate::tracker_picture::{Column, Entity, Model};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter};

pub async fn list_of_destination(db: &impl ConnectionTrait, task_id: Id, destination: &str) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).filter(Column::Destination.eq(destination)).all(db).await?)
}

/// Remembers an upload, or replaces the reference a push of the same picture kept meanwhile.
pub async fn save(db: &impl ConnectionTrait, picture: Model) -> Result<(), RekallError> {
    picture.validate()?;
    Entity::insert(picture.into_active_model())
        .on_conflict(
            OnConflict::columns([Column::TaskId, Column::Destination, Column::PicturePath])
                .update_columns([Column::RemoteReference, Column::CreatedAt])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/tracker/repository/tracker_picture_repository_tests.rs"]
mod tests;
