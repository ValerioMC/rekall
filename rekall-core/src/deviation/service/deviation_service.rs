use rekall_kernel::{RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{task_deviation, task_step};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use crate::deviation::rule::planned_step_rule::{planned_count, step_at};
use crate::deviation::{DeviationStreamEvent, Written};
use crate::{in_write, load, text_rule, Ctx};

const BODY_MAX_CHARACTERS: usize = task_deviation::BODY_MAX_CHARACTERS;

/// Writes a session's deviation on one task, or rewrites the one it already carries for that step.
#[derive(Clone)]
pub struct DeviationService {
    ctx: Ctx,
}

impl DeviationService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn write(&self, project_label: Option<&str>, task_label: &str, step_number: i32, body: Option<&str>) -> Result<Written> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let step = planned_step(&load::steps_of(tx.db(), task.id).await?, step_number)?;
            let text = validated_body(body)?;
            let now = self.ctx.now();
            let existing = repo::task_deviation::find_of_step(tx.db(), task.id, step_number).await?;
            let replaced = existing.is_some();
            match existing {
                Some(before) => {
                    let after = task_deviation::Model { body_markdown: text, ..before.clone() };
                    repo::update_if_changed(tx.db(), &before, after, now).await?;
                }
                None => {
                    let deviation = task_deviation::Model::new(task.id, step_number, text, now);
                    deviation.validate()?;
                    deviation.into_active_model().insert(tx.db()).await?;
                }
            }
            let deviations_on_task = repo::task_deviation::list_of_task(tx.db(), task.id).await?.len();
            tx.publish(DeviationStreamEvent { task_id: task.id, step_number });
            Ok(Written {
                task_anchor: task.anchor(),
                step_number,
                step_title: step.title,
                replaced,
                deviations_on_task,
            })
        })
    }
}

fn planned_step(steps: &[task_step::Model], step_number: i32) -> Result<task_step::Model> {
    step_at(steps, step_number).cloned().ok_or_else(|| step_refusal(step_number, planned_count(steps)))
}

fn step_refusal(step_number: i32, planned: usize) -> RekallError {
    RekallError::illegal(format!(
        "A deviation refers to a step of the plan, and this task's checklist has {planned} step{}: there is no step \
         {step_number}. Nothing was written.",
        if planned == 1 { "" } else { "s" }
    ))
}

fn validated_body(body: Option<&str>) -> Result<String> {
    let body = text_rule::present(body).ok_or_else(|| RekallError::illegal("A deviation needs a body: what was planned, what was built, why."))?;
    text_rule::within(body, BODY_MAX_CHARACTERS, |length| {
        format!("A deviation is capped at {BODY_MAX_CHARACTERS} characters and this one is {length}. Shorten it.")
    })
}
