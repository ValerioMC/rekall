use rekall_common::{Id, RekallError};
use rekall_model::project;
use rekall_model::task::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

use super::{at_most_one, contains_ignore_case};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// The task by id, or `NotFound` when there is none.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Task", id))
}

pub async fn find_by_ids(db: &impl ConnectionTrait, ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(Entity::find().filter(Column::Id.is_in(ids)).all(db).await?)
}

/// Every task, in creation order.
pub async fn list_oldest_first(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::CreatedAt).order_by_asc(Column::Id).all(db).await?)
}

/// Tasks whose label is `label`, ignoring case, oldest first. Labels are stored lowercase, so
/// this is a plain comparison in SQL.
pub async fn find_by_label(db: &impl ConnectionTrait, label: &str) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::Label.eq(label.to_lowercase()))
        .order_by_asc(Column::CreatedAt)
        .order_by_asc(Column::Id)
        .all(db)
        .await?)
}

/// The task `project:<label> task:<label>` names, both ignoring case. Two companies may each
/// have a project with that label; more than one match is reported, not guessed.
pub async fn find_by_anchor(db: &impl ConnectionTrait, project_label: &str, label: &str) -> Result<Option<Model>, RekallError> {
    let found = Entity::find()
        .inner_join(project::Entity)
        .filter(project::Column::Label.eq(project_label.to_lowercase()))
        .filter(Column::Label.eq(label.to_lowercase()))
        .all(db)
        .await?;
    at_most_one(found)
}

pub async fn list_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::ProjectId.eq(project_id))
        .order_by_asc(Column::Label)
        .all(db)
        .await?)
}

/// The tasks gathered under an epic, by label.
pub async fn list_children_of(db: &impl ConnectionTrait, epic_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::EpicId.eq(epic_id)).order_by_asc(Column::Label).all(db).await?)
}

/// By project label, then by label: the order the catalog lists tasks in.
pub async fn list_in_catalog_order(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .inner_join(project::Entity)
        .order_by_asc(project::Column::Label)
        .order_by_asc(Column::Label)
        .order_by_asc(Column::CreatedAt)
        .all(db)
        .await?)
}

/// Tasks whose description holds the term, newest first, at most `limit`.
pub async fn search_descriptions(db: &impl ConnectionTrait, term: &str, limit: usize) -> Result<Vec<Model>, RekallError> {
    let rows = Entity::find()
        .filter(Column::Description.is_not_null())
        .order_by_desc(Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .filter(|t| t.description.as_deref().is_some_and(|d| contains_ignore_case(d, term)))
        .take(limit)
        .collect())
}

/// Whether a task of the project other than `except` already uses this label.
pub async fn label_taken(db: &impl ConnectionTrait, project_id: Id, label: &str, except: Option<Id>) -> Result<bool, RekallError> {
    let mut query = Entity::find().filter(Column::ProjectId.eq(project_id)).filter(Column::Label.eq(label.to_lowercase()));
    if let Some(except) = except {
        query = query.filter(Column::Id.ne(except));
    }
    Ok(query.one(db).await?.is_some())
}
