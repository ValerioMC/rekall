//! What a person does to the checks that gate a claim: list them, add one for the project or for
//! a step, edit it, put a scope in order, delete one. Console-only by design.

use std::collections::HashSet;

use rekall_core::check::CheckStreamEvent;
use rekall_core::{in_read, in_write, Ctx, Tx};
use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::done_check;
use rekall_store::repository as repo;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::check::{CheckCommand, CheckOrderCommand, CheckView};

#[derive(Clone)]
pub struct CheckService {
    ctx: Ctx,
}

impl CheckService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// The project's own checks first, then each step's.
    pub async fn list(&self, project_id: Id) -> Result<Vec<CheckView>> {
        in_read!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let rows = repo::done_check::list_of_project(tx.db(), project_id).await?;
            Ok::<_, RekallError>(rows.iter().map(CheckView::of).collect())
        })
    }

    pub async fn create_for_project(&self, project_id: Id, command: CheckCommand) -> Result<CheckView> {
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            self.insert_in(&mut tx, project_id, None, command).await
        })
    }

    pub async fn create_for_step(&self, step_id: Id, command: CheckCommand) -> Result<CheckView> {
        in_write!(&self.ctx, |tx| {
            let project_id = project_of_step(&tx, step_id).await?;
            self.insert_in(&mut tx, project_id, Some(step_id), command).await
        })
    }

    pub async fn update(&self, id: Id, command: CheckCommand) -> Result<CheckView> {
        in_write!(&self.ctx, |tx| {
            let before = repo::done_check::require(tx.db(), id).await?;
            let mut check = before.clone();
            if let Some(label) = command.label.as_deref() {
                check.label = jstr::strip(label).to_string();
            }
            if let Some(text) = command.command.as_deref() {
                check.command = jstr::strip(text).to_string();
            }
            if command.folder.is_some() {
                check.folder = folder_of(command.folder.as_deref());
            }
            if let Some(seconds) = command.timeout_seconds {
                check.timeout_seconds = seconds;
            }
            if check != before {
                tx.publish(CheckStreamEvent { project_id: check.project_id });
            }
            let check = repo::update_if_changed(tx.db(), &before, check, self.ctx.now()).await?;
            Ok::<_, RekallError>(CheckView::of(&check))
        })
    }

    pub async fn reorder_for_project(&self, project_id: Id, command: CheckOrderCommand) -> Result<Vec<CheckView>> {
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            self.reorder_in(&mut tx, project_id, None, command).await
        })
    }

    pub async fn reorder_for_step(&self, step_id: Id, command: CheckOrderCommand) -> Result<Vec<CheckView>> {
        in_write!(&self.ctx, |tx| {
            let project_id = project_of_step(&tx, step_id).await?;
            self.reorder_in(&mut tx, project_id, Some(step_id), command).await
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let check = repo::done_check::require(tx.db(), id).await?;
            done_check::Entity::delete_by_id(check.id).exec(tx.db()).await?;
            tx.publish(CheckStreamEvent { project_id: check.project_id });
            Ok::<_, RekallError>(())
        })
    }

    async fn insert_in(&self, tx: &mut Tx, project_id: Id, step_id: Option<Id>, command: CheckCommand) -> Result<CheckView> {
        let existing = repo::done_check::list_of_project(tx.db(), project_id).await?;
        let now = self.ctx.now();
        let check = done_check::Model {
            id: Id::random(),
            project_id,
            step_id,
            label: command.label.as_deref().map(jstr::strip).unwrap_or_default().to_string(),
            command: command.command.as_deref().map(jstr::strip).unwrap_or_default().to_string(),
            folder: folder_of(command.folder.as_deref()),
            timeout_seconds: command.timeout_seconds.ok_or_else(|| RekallError::illegal("A check needs a timeout in seconds."))?,
            position: next_position(&existing, step_id),
            created_at: now,
            updated_at: now,
        };
        check.validate()?;
        check.clone().into_active_model().insert(tx.db()).await?;
        tx.publish(CheckStreamEvent { project_id });
        Ok(CheckView::of(&check))
    }

    /// Positions follow `check_ids`, which has to name every check of the scope once.
    async fn reorder_in(&self, tx: &mut Tx, project_id: Id, step_id: Option<Id>, command: CheckOrderCommand) -> Result<Vec<CheckView>> {
        let scope: Vec<done_check::Model> = repo::done_check::list_of_project(tx.db(), project_id).await?.into_iter().filter(|check| check.step_id == step_id).collect();
        let wanted = parse_ids(command.check_ids.as_deref().unwrap_or_default())?;
        let known: HashSet<Id> = scope.iter().map(|check| check.id).collect();
        if wanted.len() != known.len() || wanted.iter().copied().collect::<HashSet<_>>() != known {
            return Err(RekallError::illegal("An order has to name every check of the scope exactly once."));
        }
        let now = self.ctx.now();
        let mut ordered = Vec::with_capacity(scope.len());
        for (position, id) in wanted.iter().enumerate() {
            let mut check = scope.iter().find(|check| check.id == *id).cloned().expect("checked against the scope's checks");
            if check.position != position as i32 {
                check.position = position as i32;
                check.updated_at = now;
                check.clone().into_active_model().reset_all().update(tx.db()).await?;
            }
            ordered.push(CheckView::of(&check));
        }
        tx.publish(CheckStreamEvent { project_id });
        Ok(ordered)
    }
}

async fn project_of_step(tx: &Tx, step_id: Id) -> Result<Id> {
    let step = repo::task_step::require(tx.db(), step_id).await?;
    Ok(repo::task::require(tx.db(), step.task_id).await?.project_id)
}

fn folder_of(folder: Option<&str>) -> Option<String> {
    folder.filter(|text| !jstr::is_blank(text)).map(|text| jstr::strip(text).to_string())
}

fn next_position(existing: &[done_check::Model], step_id: Option<Id>) -> i32 {
    existing.iter().filter(|check| check.step_id == step_id).map(|check| check.position + 1).max().unwrap_or(0)
}

fn parse_ids(raw: &[String]) -> Result<Vec<Id>> {
    raw.iter().map(|text| text.parse::<Id>().map_err(|_| RekallError::illegal(format!("'{text}' is not a check id.")))).collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/check/service/check_service_tests.rs"]
mod tests;
