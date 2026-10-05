//! Reads the services share: a task by id or by its anchors, and what hangs off a task.

use rekall_common::{Id, InternalFailure, RekallError, Result};
use rekall_model::{project, task, task_step, wrapup};
use rekall_repository::repository as repo;
use sea_orm::ConnectionTrait;

/// The task by id, or `UnknownAnchor`: the MCP tools report a missing task as an anchor that resolves to nothing.
pub async fn task_or_unknown(db: &impl ConnectionTrait, id: Id) -> Result<task::Model> {
    repo::task::find_by_id(db, id)
        .await?
        .ok_or_else(|| RekallError::unknown_anchor(format!("No task with id {id}")))
}

pub async fn project_of(db: &impl ConnectionTrait, task: &task::Model) -> Result<project::Model> {
    repo::project::find_by_id(db, task.project_id)
        .await?
        .ok_or_else(|| RekallError::internal(InternalFailure::Integrity, format!("Task {} has no project", task.id)))
}

pub async fn steps_of(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<task_step::Model>> {
    repo::task_step::list_of_task(db, task_id).await
}

pub async fn wrapup_of(db: &impl ConnectionTrait, task_id: Id) -> Result<Option<wrapup::Model>> {
    repo::wrapup::find_of_task(db, task_id).await
}

/// Whether the task carries its own review line: only while it has no step past `DRAFT`.
pub async fn review_active(db: &impl ConnectionTrait, task_id: Id) -> Result<bool> {
    let steps = steps_of(db, task_id).await?;
    Ok(rekall_model::task::review_active(steps.iter().map(|s| &s.state)))
}

/// How every write path resolves `project:<label> task:<label>`, or a bare task label that has
/// to be unique across projects.
pub async fn resolve_task(db: &impl ConnectionTrait, project_label: Option<&str>, task_label: &str) -> Result<task::Model> {
    if let Some(project_label) = project_label {
        return repo::task::find_by_anchor(db, project_label, task_label)
            .await?
            .ok_or_else(|| {
                RekallError::unknown_anchor(format!("No task '{task_label}' on project '{project_label}'"))
            });
    }
    let mut found = repo::task::find_by_label(db, task_label).await?;
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
