use rekall_kernel::{Id, RekallError};
use crate::working_set_task::{ActiveModel, Column, Entity, Model};
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryOrder, Set};

/// The open tasks in the order they were opened.
pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::Position).all(db).await?)
}

pub async fn find_by_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(task_id).one(db).await?)
}

/// Opens the task at the end of the set. A task already open keeps its place; one that does not
/// exist is a 404.
pub async fn open(db: &impl ConnectionTrait, task_id: Id) -> Result<Model, RekallError> {
    if let Some(open) = find_by_task(db, task_id).await? {
        return Ok(open);
    }
    crate::repository::task::require(db, task_id).await?;
    let opened = Model { task_id, position: Entity::find().count(db).await? as i32 };
    opened.validate()?;
    opened.clone().into_active_model().insert(db).await?;
    Ok(opened)
}

/// Closes the task and closes the gap it leaves. A task that is not open is left alone.
pub async fn close(db: &impl ConnectionTrait, task_id: Id) -> Result<(), RekallError> {
    if find_by_task(db, task_id).await?.is_none() {
        return Ok(());
    }
    Entity::delete_by_id(task_id).exec(db).await?;
    renumber(db).await
}

async fn renumber(db: &impl ConnectionTrait) -> Result<(), RekallError> {
    for (position, open) in list(db).await?.into_iter().enumerate() {
        let position = position as i32;
        if open.position != position {
            ActiveModel { task_id: Set(open.task_id), position: Set(position) }.update(db).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/workingset/repository/working_set_task_repository_tests.rs"]
mod tests;
