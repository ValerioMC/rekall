use rekall_kernel::{Id, RekallError};
use crate::document::{Column, Entity, Model};
use crate::task;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set};

use crate::repository::contains_ignore_case;

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// The note by id, or `NotFound` when there is none.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Document", id))
}

/// A task's notes in their order on it.
pub async fn documents_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .order_by_asc(Column::Position)
        .all(db)
        .await?)
}

/// A task's notes by title.
pub async fn notes_of_task_by_title(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    let mut docs = documents_of_task(db, task_id).await?;
    docs.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(docs)
}

/// The place a task's next note takes: the end of its notes.
pub async fn next_position(db: &impl ConnectionTrait, task_id: Id) -> Result<i32, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).count(db).await? as i32)
}

/// Closes the gap a deleted note leaves, so a task's positions stay dense from zero.
pub async fn close_gaps(db: &impl ConnectionTrait, task_id: Id) -> Result<(), RekallError> {
    for (at, note) in documents_of_task(db, task_id).await?.into_iter().enumerate() {
        if note.position != at as i32 {
            let mut moved = note.into_active_model();
            moved.position = Set(at as i32);
            moved.update(db).await?;
        }
    }
    Ok(())
}

pub async fn list_newest_first(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_desc(Column::UpdatedAt).all(db).await?)
}

/// Every note on any task of the project, newest first.
pub async fn list_on_project_tasks(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    let task_ids: Vec<Id> = task::Entity::find()
        .select_only()
        .column(task::Column::Id)
        .filter(task::Column::ProjectId.eq(project_id))
        .into_tuple()
        .all(db)
        .await?;
    Ok(Entity::find()
        .filter(Column::TaskId.is_in(task_ids))
        .order_by_desc(Column::UpdatedAt)
        .all(db)
        .await?)
}

/// Title or body holds the term, ignoring case, by title.
pub async fn search_by_title(db: &impl ConnectionTrait, term: &str) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .order_by_asc(Column::Title)
        .all(db)
        .await?
        .into_iter()
        .filter(|d| contains_ignore_case(&d.title, term) || contains_ignore_case(&d.body_markdown, term))
        .collect())
}

/// Title or body holds the term literally, newest first.
pub async fn search_newest_first(db: &impl ConnectionTrait, term: &str, limit: usize) -> Result<Vec<Model>, RekallError> {
    Ok(list_newest_first(db)
        .await?
        .into_iter()
        .filter(|d| contains_ignore_case(&d.title, term) || contains_ignore_case(&d.body_markdown, term))
        .take(limit)
        .collect())
}

/// Notes whose id starts with the prefix: how a `note:` anchor resolves.
pub async fn find_by_anchor_prefix(db: &impl ConnectionTrait, prefix: &str) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::Id.starts_with(prefix))
        .all(db)
        .await?)
}

