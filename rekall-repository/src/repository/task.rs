use rekall_common::{Id, RekallError};
use rekall_model::project;
use rekall_model::task::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

use super::{at_most_one, contains_ignore_case, equals_ignore_case};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn find_by_ids(db: &impl ConnectionTrait, ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(Entity::find().filter(Column::Id.is_in(ids)).all(db).await?)
}

pub async fn find_all(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::CreatedAt).order_by_asc(Column::Id).all(db).await?)
}

pub async fn find_by_label_ignore_case(db: &impl ConnectionTrait, label: &str) -> Result<Vec<Model>, RekallError> {
    Ok(find_all(db).await?.into_iter().filter(|t| equals_ignore_case(&t.label, label)).collect())
}

pub async fn find_by_project_label_ignore_case_and_label_ignore_case(
    db: &impl ConnectionTrait,
    project_label: &str,
    label: &str,
) -> Result<Option<Model>, RekallError> {
    let projects: Vec<Id> = super::project::find_by_label_ignore_case(db, project_label)
        .await?
        .into_iter()
        .map(|p| p.id)
        .collect();
    let found = find_by_label_ignore_case(db, label)
        .await?
        .into_iter()
        .filter(|t| projects.contains(&t.project_id))
        .collect();
    at_most_one(found)
}

pub async fn find_by_project_id_order_by_label_asc(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::ProjectId.eq(project_id))
        .order_by_asc(Column::Label)
        .all(db)
        .await?)
}

/// `findAllByOrderByProjectLabelAscLabelAsc`: grouped by project label, then by label.
pub async fn find_all_by_order_by_project_label_asc_label_asc(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    let projects = project::Entity::find().all(db).await?;
    let mut tasks = Entity::find().all(db).await?;
    let label_of = |id: &Id| projects.iter().find(|p| p.id == *id).map(|p| p.label.clone()).unwrap_or_default();
    tasks.sort_by(|a, b| (label_of(&a.project_id), &a.label).cmp(&(label_of(&b.project_id), &b.label)));
    Ok(tasks)
}

/// `searchDescriptions`: tasks whose description holds the term, newest first, at most `limit`.
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
