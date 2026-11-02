use rekall_store::{step_link, step_visit, task_step, TaskStepState};
use serde::Serialize;

use crate::flow::{carries_routes, flow_has_ended};

/// Where a step stands for someone watching the work. `NotNeeded` is an open step of a flow that
/// reached its end without entering it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StepProgressState {
    Open,
    Running,
    Claimed,
    Done,
    NotNeeded,
}

/// One released step of a task as the work reads it; `current` marks the one being worked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepProgress {
    pub title: String,
    pub state: StepProgressState,
    pub current: bool,
}

/// The released steps in checklist order. The current one is where the flow stands, else the
/// step a session is running, else the first open one; none once the task is complete.
pub fn progress_of(steps: &[task_step::Model], visits: &[step_visit::Model], links: &[step_link::Model]) -> Vec<StepProgress> {
    let released: Vec<task_step::Model> = steps.iter().filter(|step| !step.state.draft()).cloned().collect();
    let ended = carries_routes(links, &released) && flow_has_ended(&released, visits);
    let current = if ended { None } else { current_step(&released, visits) };
    released
        .iter()
        .map(|step| StepProgress {
            title: step.title.clone(),
            state: state_of(step, ended, visits),
            current: current == Some(step.id),
        })
        .collect()
}

fn current_step(released: &[task_step::Model], visits: &[step_visit::Model]) -> Option<rekall_kernel::Id> {
    let standing = visits.iter().find(|visit| visit.is_open()).map(|visit| visit.step_id);
    standing
        .or_else(|| released.iter().find(|step| step.state.running()).map(|step| step.id))
        .or_else(|| released.iter().find(|step| step.state == TaskStepState::Open).map(|step| step.id))
}

fn state_of(step: &task_step::Model, flow_ended: bool, visits: &[step_visit::Model]) -> StepProgressState {
    match step.state {
        TaskStepState::Running => StepProgressState::Running,
        TaskStepState::Claimed => StepProgressState::Claimed,
        TaskStepState::Done => StepProgressState::Done,
        _ if flow_ended && !visits.iter().any(|visit| visit.step_id == step.id) => StepProgressState::NotNeeded,
        _ => StepProgressState::Open,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/flow/model/step_progress_tests.rs"]
mod tests;
