use rekall_kernel::{Id, RekallError};
use crate::project;
use crate::enums::TaskStepState;
use crate::task::{Column, Entity, Model};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Set};

use crate::repository::{at_most_one, contains_ignore_case};
use crate::task_key::TaskKey;

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

/// The task whose code is `code`, ignoring case: `rk-12` finds `RK-12`.
pub async fn find_by_code(db: &impl ConnectionTrait, code: &str) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find().filter(Column::Code.eq(code.to_ascii_uppercase())).one(db).await?)
}

/// The task `project:<label> task:<label>` names, both ignoring case: how an anchor written before
/// tasks had codes still finds its task.
pub async fn find_by_project_and_label(db: &impl ConnectionTrait, project_label: &str, label: &str) -> Result<Option<Model>, RekallError> {
    let found = Entity::find()
        .inner_join(project::Entity)
        .filter(project::Column::Label.eq(project_label.to_lowercase()))
        .filter(Column::Label.eq(label.to_lowercase()))
        .all(db)
        .await?;
    at_most_one(found)
}

/// The number the next task of the project takes: one past the highest it holds.
pub async fn next_number(db: &impl ConnectionTrait, project_id: Id) -> Result<i32, RekallError> {
    let highest = Entity::find().filter(Column::ProjectId.eq(project_id)).order_by_desc(Column::Number).one(db).await?;
    Ok(highest.map_or(1, |task| task.number + 1))
}

/// Gives a new task the next number of its project, the code that goes with it and, as its internal
/// label (the name of its branch and worktree), that code in lowercase.
pub async fn assign_next_code(db: &impl ConnectionTrait, task: &mut Model, project: &project::Model) -> Result<(), RekallError> {
    let number = next_number(db, project.id).await?;
    task.assign_code(&project.task_key, number);
    task.label = task.code.to_ascii_lowercase();
    Ok(())
}

/// Writes every code of the project again from its key, for when the key changes.
pub async fn recode_project(db: &impl ConnectionTrait, project_id: Id, project_key: &str) -> Result<(), RekallError> {
    for task in Entity::find().filter(Column::ProjectId.eq(project_id)).all(db).await? {
        let mut recoded = task.clone().into_active_model();
        recoded.code = Set(TaskKey::code(project_key, task.number));
        recoded.update(db).await?;
    }
    Ok(())
}

pub async fn list_of_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::ProjectId.eq(project_id))
        .order_by_asc(Column::Number)
        .all(db)
        .await?)
}

/// The tasks that have a git worktree of their own, by label.
pub async fn list_with_worktree(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::WorktreeFolder.is_not_null()).order_by_asc(Column::Label).all(db).await?)
}

/// The tasks gathered under an epic, by label.
pub async fn list_children_of(db: &impl ConnectionTrait, epic_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().filter(Column::EpicId.eq(epic_id)).order_by_asc(Column::Label).all(db).await?)
}

/// By project label, then by number: the order the catalog lists tasks in.
pub async fn list_in_catalog_order(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .inner_join(project::Entity)
        .order_by_asc(project::Column::Label)
        .order_by_asc(Column::Number)
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

