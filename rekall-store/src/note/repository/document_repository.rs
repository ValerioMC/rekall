use rekall_kernel::{Id, RekallError};
use crate::document::{Column, Entity, Model};
use crate::{document_task, task};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::repository::contains_ignore_case;

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn find_by_ids(db: &impl ConnectionTrait, ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(Entity::find().filter(Column::Id.is_in(ids)).all(db).await?)
}

/// The links of one task, in `position` order.
pub async fn links_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<document_task::Model>, RekallError> {
    Ok(document_task::Entity::find()
        .filter(document_task::Column::TaskId.eq(task_id))
        .order_by_asc(document_task::Column::Position)
        .all(db)
        .await?)
}

/// The links of one note in the order its tasks were attached.
pub async fn links_of_document(db: &impl ConnectionTrait, document_id: Id) -> Result<Vec<document_task::Model>, RekallError> {
    Ok(document_task::Entity::find()
        .filter(document_task::Column::DocumentId.eq(document_id))
        .order_by_asc(sea_orm::sea_query::Expr::cust("rowid"))
        .all(db)
        .await?)
}

/// A task's notes in their order on it.
pub async fn documents_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    let links = links_of_task(db, task_id).await?;
    let docs = find_by_ids(db, links.iter().map(|l| l.document_id).collect()).await?;
    Ok(links
        .iter()
        .filter_map(|l| docs.iter().find(|d| d.id == l.document_id).cloned())
        .collect())
}

/// The tasks a note is on, in the order they were attached.
pub async fn tasks_of_document(db: &impl ConnectionTrait, document_id: Id) -> Result<Vec<task::Model>, RekallError> {
    let links = links_of_document(db, document_id).await?;
    let tasks = crate::repository::task::find_by_ids(db, links.iter().map(|l| l.task_id).collect()).await?;
    Ok(links
        .iter()
        .filter_map(|l| tasks.iter().find(|t| t.id == l.task_id).cloned())
        .collect())
}

/// A task's notes by title.
pub async fn notes_of_task_by_title(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    let mut docs = documents_of_task(db, task_id).await?;
    docs.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(docs)
}

pub async fn list_newest_first(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_desc(Column::UpdatedAt).all(db).await?)
}

/// Every note on any task of the project, once, newest first.
pub async fn list_on_project_tasks(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    let task_ids: Vec<Id> = task::Entity::find()
        .select_only()
        .column(task::Column::Id)
        .filter(task::Column::ProjectId.eq(project_id))
        .into_tuple()
        .all(db)
        .await?;
    let document_ids: Vec<Id> = document_task::Entity::find()
        .select_only()
        .column(document_task::Column::DocumentId)
        .filter(document_task::Column::TaskId.is_in(task_ids))
        .distinct()
        .into_tuple()
        .all(db)
        .await?;
    Ok(Entity::find()
        .filter(Column::Id.is_in(document_ids))
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

