use rekall_core::{in_read, Ctx, Tx};
use rekall_kernel::{Id, Result};
use rekall_store::repository as repo;
use rekall_store::{task, task_deviation, task_step, TaskStepState};

use crate::adherence::{AdherenceCounts, PlanAdherenceView, ProjectAdherenceView, StepVisitsView, TaskSentBackView};

const MOST_SENT_BACK_LISTED: usize = 5;

#[derive(Clone)]
pub struct PlanAdherenceService {
    ctx: Ctx,
}

impl PlanAdherenceService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// A task with no steps gives a valid view of zeros.
    pub async fn of_task(&self, task_id: Id) -> Result<PlanAdherenceView> {
        in_read!(&self.ctx, |tx| {
            repo::task::require(tx.db(), task_id).await?;
            adherence_of(&tx, task_id).await
        })
    }

    pub async fn of_project(&self, project_id: Id) -> Result<ProjectAdherenceView> {
        in_read!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let tasks = repo::task::list_of_project(tx.db(), project_id).await?;
            let mut counts = AdherenceCounts::default();
            let mut deviations = 0;
            let mut sent_back = Vec::new();
            for task in &tasks {
                let adherence = adherence_of(&tx, task.id).await?;
                counts = counts.plus(adherence.counts);
                deviations += adherence.deviation_steps.len();
                sent_back.push(sent_back_of(task, &adherence));
            }
            Ok(ProjectAdherenceView { counts, deviations, most_sent_back: most_sent_back(sent_back) })
        })
    }
}

async fn adherence_of(tx: &Tx, task_id: Id) -> Result<PlanAdherenceView> {
    let steps = planned_steps(repo::task_step::list_of_task(tx.db(), task_id).await?);
    let deviations = repo::task_deviation::list_of_task(tx.db(), task_id).await?;
    let visits = repo::step_visit::list_of_task(tx.db(), task_id).await?;
    Ok(PlanAdherenceView {
        counts: AdherenceCounts::of(&steps)?,
        deviation_steps: deviation_steps(&deviations),
        visits: StepVisitsView::of_flow(&steps, &visits),
    })
}

fn deviation_steps(deviations: &[task_deviation::Model]) -> Vec<usize> {
    deviations.iter().filter_map(|deviation| usize::try_from(deviation.step_number).ok()).collect()
}

fn planned_steps(steps: Vec<task_step::Model>) -> Vec<task_step::Model> {
    steps.into_iter().filter(|step| step.state != TaskStepState::Draft).collect()
}

fn sent_back_of(task: &task::Model, adherence: &PlanAdherenceView) -> TaskSentBackView {
    TaskSentBackView {
        task_id: task.id,
        code: task.code.clone(),
        title: task.title.clone(),
        sent_back: adherence.counts.sent_back,
        passes_total: adherence.counts.passes_total,
    }
}

fn most_sent_back(mut tasks: Vec<TaskSentBackView>) -> Vec<TaskSentBackView> {
    tasks.retain(|task| task.sent_back > 0);
    tasks.sort_by(|a, b| b.sent_back.cmp(&a.sent_back).then_with(|| a.code.cmp(&b.code)));
    tasks.truncate(MOST_SENT_BACK_LISTED);
    tasks
}

#[cfg(test)]
#[path = "../../../tests/unit/adherence/service/plan_adherence_service_tests.rs"]
mod tests;
