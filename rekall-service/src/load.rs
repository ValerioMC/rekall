//! Reads the services share: a task by id or by its anchors, a project by its label, a step by its
//! number or title, and what hangs off a task.

use std::sync::LazyLock;

use regex::Regex;
use rekall_common::{jstr, Id, InternalFailure, RekallError, Result};
use rekall_store::{project, task, task_step, wrapup};
use rekall_store::repository as repo;
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
    Ok(rekall_store::task::review_active(steps.iter().map(|s| &s.state)))
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
            candidates.push(project_of(db, task).await?.anchor());
        }
        return Err(RekallError::ambiguous(task_label, candidates));
    }
    Ok(found.remove(0))
}

/// The project `project:<label>` names. Labels are unique within a company only, so a label two
/// companies share is ambiguous, and the candidates say which is which.
pub async fn resolve_project(db: &impl ConnectionTrait, label: &str) -> Result<project::Model> {
    let mut found = repo::project::find_by_label(db, label).await?;
    match found.len() {
        0 => Err(RekallError::unknown_anchor(format!("No project matches '{label}'"))),
        1 => Ok(found.remove(0)),
        _ => Err(RekallError::ambiguous(label, found.iter().map(|project| format!("{} ({})", project.anchor(), project.title)).collect())),
    }
}

/// A step by its one-based number in the checklist, or by its exact title.
pub async fn resolve_step(db: &impl ConnectionTrait, task_id: Id, reference: Option<&str>) -> Result<task_step::Model> {
    static DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").expect("a valid pattern"));
    let ordered = steps_of(db, task_id).await?;
    if ordered.is_empty() {
        return Err(RekallError::illegal("That task has no steps to move."));
    }
    let trimmed = reference.map(jstr::strip).unwrap_or("");
    if DIGITS.is_match(trimmed) {
        return step_by_number(ordered, trimmed);
    }
    let by_title: Vec<&task_step::Model> = ordered
        .iter()
        .filter(|step| jstr::equals_ignore_case(jstr::strip(&step.title), trimmed))
        .collect();
    if by_title.len() == 1 {
        return Ok(by_title[0].clone());
    }
    let titles: Vec<String> = ordered
        .iter()
        .enumerate()
        .map(|(at, step)| format!("{}. {}", at + 1, step.title))
        .collect();
    Err(RekallError::illegal(format!(
        "No step matches '{}'. Pass its number or its exact title:\n{}",
        reference.unwrap_or("null"),
        titles.join("\n")
    )))
}

/// `number` is all digits; one too large to parse is out of range like any other.
fn step_by_number(mut ordered: Vec<task_step::Model>, number: &str) -> Result<task_step::Model> {
    let count = ordered.len();
    match number.parse::<usize>() {
        Ok(one_based) if (1..=count).contains(&one_based) => Ok(ordered.swap_remove(one_based - 1)),
        _ => Err(RekallError::illegal(format!("There is no step {number}. The checklist has {count}."))),
    }
}
