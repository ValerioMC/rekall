use rekall_kernel::{Id, RekallError};
use crate::working_set_place::{ActiveModel, Column, Entity, Model};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set};

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::ProjectId).all(db).await?)
}

pub async fn find_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find().filter(Column::ProjectId.eq(project_id)).one(db).await?)
}

/// Replaces the place remembered for the project: one of its tasks, or its task list for `None`.
/// A project that does not exist is a 404.
pub async fn remember(db: &impl ConnectionTrait, project_id: Id, task_id: Option<Id>) -> Result<Model, RekallError> {
    crate::repository::project::require(db, project_id).await?;
    let place = Model { project_id, task_id };
    match find_of_project(db, project_id).await? {
        Some(_) => {
            ActiveModel { project_id: Set(project_id), task_id: Set(task_id) }.update(db).await?;
        }
        None => {
            place.clone().into_active_model().insert(db).await?;
        }
    }
    Ok(place)
}

#[cfg(test)]
#[path = "../../../tests/unit/workingset/repository/working_set_place_repository_tests.rs"]
mod tests;
