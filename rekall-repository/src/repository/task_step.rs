use rekall_common::{Id, RekallError};
use rekall_model::task_step::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

use super::contains_ignore_case;

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

/// The step by id, or `NotFound` when there is none.
pub async fn require(db: &impl ConnectionTrait, id: Id) -> Result<Model, RekallError> {
    find_by_id(db, id).await?.ok_or_else(|| RekallError::not_found("Step", id))
}

pub async fn list_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .order_by_asc(Column::Position)
        .all(db)
        .await?)
}

pub async fn list_of_tasks(db: &impl ConnectionTrait, task_ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    if task_ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(Entity::find()
        .filter(Column::TaskId.is_in(task_ids))
        .order_by_asc(Column::Position)
        .all(db)
        .await?)
}

pub async fn list_all_in_order(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .order_by_asc(Column::TaskId)
        .order_by_asc(Column::Position)
        .all(db)
        .await?)
}

/// `search`: steps whose title or detail holds the (escaped, literal) term, newest first.
pub async fn search(db: &impl ConnectionTrait, term: &str, limit: usize) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .order_by_desc(Column::UpdatedAt)
        .all(db)
        .await?
        .into_iter()
        .filter(|s| {
            contains_ignore_case(&s.title, term)
                || s.body_markdown.as_deref().is_some_and(|b| contains_ignore_case(b, term))
        })
        .take(limit)
        .collect())
}
