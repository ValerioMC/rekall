//! Reads the services share: an entity by id, or the refusal the Java code threw when it was
//! not there, and a task resolved by its anchors.

use rekall_common::{Id, RekallError, Result};
use rekall_model::{project, task, task_step, wrapup};
use rekall_repository::repository as repo;
use sea_orm::ConnectionTrait;

/// `tasks.findById(id).orElseThrow(() -> new UnknownAnchorException("No task with id " + id))`.
pub async fn task_or_unknown(db: &impl ConnectionTrait, id: Id) -> Result<task::Model> {
    repo::task::find_by_id(db, id)
        .await?
        .ok_or_else(|| RekallError::unknown_anchor(format!("No task with id {id}")))
}

pub async fn project_of(db: &impl ConnectionTrait, task: &task::Model) -> Result<project::Model> {
    repo::project::find_by_id(db, task.project_id)
        .await?
        .ok_or_else(|| RekallError::internal("IllegalStateException", format!("Task {} has no project", task.id)))
}

pub async fn steps_of(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<task_step::Model>> {
    repo::task_step::find_by_task_id_order_by_position_asc(db, task_id).await
}

pub async fn wrapup_of(db: &impl ConnectionTrait, task_id: Id) -> Result<Option<wrapup::Model>> {
    repo::wrapup::find_by_task_id(db, task_id).await
}

/// `Task.reviewActive()`, reading the task's steps.
pub async fn review_active(db: &impl ConnectionTrait, task_id: Id) -> Result<bool> {
    let steps = steps_of(db, task_id).await?;
    Ok(rekall_model::task::review_active(steps.iter().map(|s| &s.state)))
}

/// How every write path resolves `project:<label> task:<label>` (or a bare task label that has
/// to be unique across projects): the private `resolveTask` each Java write service carried.
pub async fn resolve_task(db: &impl ConnectionTrait, project_label: Option<&str>, task_label: &str) -> Result<task::Model> {
    if let Some(project_label) = project_label {
        return repo::task::find_by_project_label_ignore_case_and_label_ignore_case(db, project_label, task_label)
            .await?
            .ok_or_else(|| {
                RekallError::unknown_anchor(format!("No task '{task_label}' on project '{project_label}'"))
            });
    }
    let mut found = repo::task::find_by_label_ignore_case(db, task_label).await?;
    if found.is_empty() {
        return Err(RekallError::unknown_anchor(format!("No task matches '{task_label}'")));
    }
    if found.len() > 1 {
        let mut candidates = Vec::with_capacity(found.len());
        for task in &found {
            candidates.push(format!("project:{}", project_of(db, task).await?.label));
        }
        return Err(RekallError::ambiguous(task_label, candidates));
    }
    Ok(found.remove(0))
}
