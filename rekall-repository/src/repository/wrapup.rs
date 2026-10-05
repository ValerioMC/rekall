use rekall_common::{Id, RekallError};
use rekall_model::wrapup::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

use super::contains_ignore_case;

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn find_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).one(db).await?)
}

pub async fn list_of_tasks(db: &impl ConnectionTrait, task_ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    if task_ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(Entity::find().filter(Column::TaskId.is_in(task_ids)).all(db).await?)
}

pub async fn list_newest_first(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_desc(Column::UpdatedAt).all(db).await?)
}

/// `search`: wrapups whose body holds the (escaped, literal) term, newest first.
pub async fn search(db: &impl ConnectionTrait, term: &str, limit: usize) -> Result<Vec<Model>, RekallError> {
    Ok(list_newest_first(db)
        .await?
        .into_iter()
        .filter(|w| contains_ignore_case(&w.body_markdown, term))
        .take(limit)
        .collect())
}
