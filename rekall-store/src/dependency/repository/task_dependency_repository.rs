use rekall_kernel::{Id, RekallError};
use crate::task_dependency::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// A lookup that must exist: the dependency, or a 404.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Dependency", id))
}

/// What `task_id` waits for, in the order it was added.
pub async fn list_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).order_by_asc(Column::CreatedAt).all(db).await?)
}

/// Every dependency whose waiting task is one of `task_ids`.
pub async fn list_of_tasks(db: &impl ConnectionTrait, task_ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.is_in(task_ids)).order_by_asc(Column::CreatedAt).all(db).await?)
}

/// Every dependency among the tasks of one project, for walking the graph.
pub async fn list_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    let tasks = crate::repository::task::list_of_project(db, project_id).await?;
    list_of_tasks(db, tasks.into_iter().map(|task| task.id).collect()).await
}

pub async fn find_pair(db: &impl ConnectionTrait, task_id: Id, depends_on_task_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .filter(Column::DependsOnTaskId.eq(depends_on_task_id))
        .one(db)
        .await?)
}
