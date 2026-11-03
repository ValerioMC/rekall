//! Runs the checks that apply to a step and keeps what each run found. The rows are read in a
//! transaction of their own and released before the first command starts, since a command may take
//! minutes; each result is written as soon as its run ends.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{done_check, done_check_result};
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use tracing::info;

use crate::check::rule::working_folder::working_folder;
use crate::check::{CheckOutcome, CheckRan, CheckReport, CheckRun, CheckRunner};
use crate::{in_read, in_write, Ctx};

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
    checks: Vec<PlannedCheck>,
}

impl CheckExecutionService {
    pub fn new(ctx: Ctx, runner: Arc<dyn CheckRunner>) -> Self {
        Self { ctx, runner }
    }

    /// The project's checks, then the step's, in order and one at a time, stopping at the first that
    /// fails. Every run that happened is kept, the failing one included.
    pub async fn run_all(&self, step_id: Id) -> Result<CheckReport> {
        let plan = self.plan_for(step_id).await?;
        let mut passed = Vec::with_capacity(plan.checks.len());
        for planned in &plan.checks {
            let timeout = Duration::from_secs(planned.check.timeout_seconds.max(1) as u64);
            let run = self.runner.run(&planned.folder, &planned.check.command, timeout).await?;
            info!(label = %planned.check.label, exit_code = ?run.outcome.exit_code(), timed_out = run.outcome == CheckOutcome::TimedOut, duration_ms = run.duration.as_millis() as u64, "A check ran");
            self.keep(&plan, step_id, &planned.check, &run).await?;
            if !run.outcome.passed() {
                return Ok(CheckReport::Failed { label: planned.check.label.clone(), outcome: run.outcome, output_tail: run.output_tail });
            }
            passed.push(CheckRan { label: planned.check.label.clone(), duration: run.duration });
        }
        Ok(if passed.is_empty() { CheckReport::NoChecks } else { CheckReport::Passed { runs: passed } })
    }

    async fn plan_for(&self, step_id: Id) -> Result<Plan> {
        in_read!(&self.ctx, |tx| {
            let step = repo::task_step::require(tx.db(), step_id).await?;
            let task = repo::task::require(tx.db(), step.task_id).await?;
            let rows: Vec<done_check::Model> = repo::done_check::list_of_project(tx.db(), task.project_id)
                .await?
                .into_iter()
                .filter(|check| check.step_id.is_none() || check.step_id == Some(step_id))
                .collect();
            if rows.is_empty() {
                return Ok(Plan { task_id: task.id, checks: Vec::new() });
            }
            let project = repo::project::require(tx.db(), task.project_id).await?;
            let project_folder = project.repo_folder.as_deref().map(PathBuf::from).ok_or_else(|| RekallError::illegal("This project has no folder to run its checks in."))?;
            let checks = rows
                .into_iter()
                .map(|check| Ok(PlannedCheck { folder: working_folder(&project_folder, check.folder.as_deref())?, check }))
                .collect::<Result<Vec<_>>>()?;
            Ok::<_, RekallError>(Plan { task_id: task.id, checks })
        })
    }

    async fn keep(&self, plan: &Plan, step_id: Id, check: &done_check::Model, run: &CheckRun) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let result = done_check_result::Model {
                id: Id::random(),
                check_id: check.id,
                task_id: plan.task_id,
                step_id: Some(step_id),
                passed: run.outcome.passed(),
                exit_code: run.outcome.exit_code(),
                output_tail: run.output_tail.clone(),
                ran_at: self.ctx.now(),
                duration_ms: run.duration.as_millis() as i64,
            };
            result.into_active_model().insert(tx.db()).await?;
            Ok::<_, RekallError>(())
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/check/service/check_execution_service_tests.rs"]
mod tests;
