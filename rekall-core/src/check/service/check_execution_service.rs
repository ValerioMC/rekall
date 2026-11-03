//! Runs the checks that apply to a step, or to a task with no steps, and keeps what each run found.
//! The rows are read in a transaction of their own and released before the first command starts,
//! since a command may take minutes; each result is written as soon as its run ends.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{done_check, done_check_result, TaskStepState};
use sea_orm::{ActiveModelTrait, ConnectionTrait, IntoActiveModel};
use tracing::info;

use crate::check::rule::working_folder::working_folder;
use crate::check::{CheckOutcome, CheckRan, CheckReport, CheckResultStreamEvent, CheckRun, CheckRunner};
use crate::{in_read, in_write, load, Ctx};

#[derive(Clone)]
pub struct CheckExecutionService {
    ctx: Ctx,
    runner: Arc<dyn CheckRunner>,
}

struct PlannedCheck {
    check: done_check::Model,
    folder: PathBuf,
}

struct Plan {
    task_id: Id,
    step_id: Option<Id>,
    checks: Vec<PlannedCheck>,
}

impl CheckExecutionService {
    pub fn new(ctx: Ctx, runner: Arc<dyn CheckRunner>) -> Self {
        Self { ctx, runner }
    }

    /// The project's checks, then the step's, in order and one at a time, stopping at the first that
    /// fails. Every run that happened is kept, the failing one included.
    pub async fn run_all(&self, step_id: Id) -> Result<CheckReport> {
        let plan = self.plan_for_step(step_id).await?;
        self.run(plan).await
    }

    /// The checks a session's claim of a step must pass. A step that cannot be claimed (a draft, or
    /// one already accepted) runs none: the claim itself refuses it.
    pub async fn run_for_step_claim(&self, project_label: Option<&str>, task_label: &str, step_ref: Option<&str>) -> Result<CheckReport> {
        match self.claimable_step(project_label, task_label, step_ref).await? {
            Some(step_id) => self.run_all(step_id).await,
            None => Ok(CheckReport::NoChecks),
        }
    }

    /// The project's own checks, for the wrapup that claims a task with no steps. A task with steps
    /// is claimed step by step, so its wrapup runs none.
    pub async fn run_for_wrapup_claim(&self, project_label: Option<&str>, task_label: &str) -> Result<CheckReport> {
        let plan = self.plan_for_stepless_task(project_label, task_label).await?;
        self.run(plan).await
    }

    async fn run(&self, plan: Plan) -> Result<CheckReport> {
        let mut passed = Vec::with_capacity(plan.checks.len());
        for planned in &plan.checks {
            let timeout = Duration::from_secs(planned.check.timeout_seconds.max(1) as u64);
            let run = self.runner.run(&planned.folder, &planned.check.command, timeout).await?;
            info!(label = %planned.check.label, exit_code = ?run.outcome.exit_code(), timed_out = run.outcome == CheckOutcome::TimedOut, duration_ms = run.duration.as_millis() as u64, "A check ran");
            self.keep(&plan, &planned.check, &run).await?;
            if !run.outcome.passed() {
                return Ok(CheckReport::Failed { label: planned.check.label.clone(), outcome: run.outcome, output_tail: run.output_tail });
            }
            passed.push(CheckRan { label: planned.check.label.clone(), duration: run.duration });
        }
        Ok(if passed.is_empty() { CheckReport::NoChecks } else { CheckReport::Passed { runs: passed } })
    }

    async fn claimable_step(&self, project_label: Option<&str>, task_label: &str, step_ref: Option<&str>) -> Result<Option<Id>> {
        in_read!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let step = load::resolve_step(tx.db(), task.id, step_ref).await?;
            let claimable = !matches!(step.state, TaskStepState::Draft | TaskStepState::Done);
            Ok::<_, RekallError>(claimable.then_some(step.id))
        })
    }

    async fn plan_for_step(&self, step_id: Id) -> Result<Plan> {
        in_read!(&self.ctx, |tx| {
            let step = repo::task_step::require(tx.db(), step_id).await?;
            let task = repo::task::require(tx.db(), step.task_id).await?;
            plan_in(tx.db(), task.id, task.project_id, Some(step_id)).await
        })
    }

    async fn plan_for_stepless_task(&self, project_label: Option<&str>, task_label: &str) -> Result<Plan> {
        in_read!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            if !load::review_active(tx.db(), task.id).await? {
                return Ok(Plan { task_id: task.id, step_id: None, checks: Vec::new() });
            }
            plan_in(tx.db(), task.id, task.project_id, None).await
        })
    }

    async fn keep(&self, plan: &Plan, check: &done_check::Model, run: &CheckRun) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let result = done_check_result::Model {
                id: Id::random(),
                check_id: check.id,
                task_id: plan.task_id,
                step_id: plan.step_id,
                passed: run.outcome.passed(),
                exit_code: run.outcome.exit_code(),
                output_tail: run.output_tail.clone(),
                ran_at: self.ctx.now(),
                duration_ms: run.duration.as_millis() as i64,
            };
            result.into_active_model().insert(tx.db()).await?;
            tx.publish(CheckResultStreamEvent { task_id: plan.task_id });
            Ok::<_, RekallError>(())
        })
    }
}

/// The project's own checks, then the step's when there is one, each with the folder it runs in.
async fn plan_in(db: &impl ConnectionTrait, task_id: Id, project_id: Id, step_id: Option<Id>) -> Result<Plan> {
    let rows: Vec<done_check::Model> = repo::done_check::list_of_project(db, project_id)
        .await?
        .into_iter()
        .filter(|check| check.step_id.is_none() || check.step_id == step_id)
        .collect();
    if rows.is_empty() {
        return Ok(Plan { task_id, step_id, checks: Vec::new() });
    }
    let project = repo::project::require(db, project_id).await?;
    let project_folder = project.repo_folder.as_deref().map(PathBuf::from).ok_or_else(|| RekallError::illegal("This project has no folder to run its checks in."))?;
    let checks = rows
        .into_iter()
        .map(|check| Ok(PlannedCheck { folder: working_folder(&project_folder, check.folder.as_deref())?, check }))
        .collect::<Result<Vec<_>>>()?;
    Ok(Plan { task_id, step_id, checks })
}

#[cfg(test)]
#[path = "../../../tests/unit/check/service/check_execution_service_tests.rs"]
mod tests;
