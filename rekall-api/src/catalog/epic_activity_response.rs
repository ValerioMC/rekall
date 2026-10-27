use rekall_common::{Id, Instant};
use rekall_model::{commit_reference, task, task_step, wrapup};
use serde::Serialize;

use crate::catalog::EpicActivityKind;

/// The most moves an epic's dashboard shows: enough to read what moved lately, not a history.
pub const ACTIVITY_MAX: usize = 60;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpicActivityResponse {
    pub task_id: Id,
    pub kind: EpicActivityKind,
    /// The step's title or the commit's subject; `None` when the move is the task's own.
    pub subject: Option<String>,
    pub at: Instant,
}

impl EpicActivityResponse {
    /// Every moment the rows of an epic's tasks record, newest first, at most `ACTIVITY_MAX`.
    pub fn feed(
        tasks: &[task::Model],
        steps: &[task_step::Model],
        wrapups: &[wrapup::Model],
        commits: &[commit_reference::Model],
    ) -> Vec<Self> {
        let mut feed: Vec<Self> = tasks.iter().flat_map(task_moments).collect();
        feed.extend(steps.iter().flat_map(step_moments));
        feed.extend(wrapups.iter().map(|wrapup| moment(wrapup.task_id, EpicActivityKind::WrapupWritten, None, wrapup.updated_at)));
        feed.extend(commits.iter().map(|commit| {
            moment(commit.task_id, EpicActivityKind::CommitLogged, Some(commit.comment.clone()), commit.created_at)
        }));
        feed.sort_by(|left, right| right.at.cmp(&left.at));
        feed.truncate(ACTIVITY_MAX);
        feed
    }
}

fn task_moments(task: &task::Model) -> Vec<EpicActivityResponse> {
    [(EpicActivityKind::TaskClaimed, task.claimed_at), (EpicActivityKind::TaskAccepted, task.accepted_at)]
        .into_iter()
        .filter_map(|(kind, at)| at.map(|at| moment(task.id, kind, None, at)))
        .collect()
}

fn step_moments(step: &task_step::Model) -> Vec<EpicActivityResponse> {
    [
        (EpicActivityKind::StepStarted, step.running_at),
        (EpicActivityKind::StepClaimed, step.claimed_at),
        (EpicActivityKind::StepAccepted, step.done_at),
    ]
    .into_iter()
    .filter_map(|(kind, at)| at.map(|at| moment(step.task_id, kind, Some(step.title.clone()), at)))
    .collect()
}

fn moment(task_id: Id, kind: EpicActivityKind, subject: Option<String>, at: Instant) -> EpicActivityResponse {
    EpicActivityResponse { task_id, kind, subject, at }
}

#[cfg(test)]
#[path = "../../tests/unit/catalog/epic_activity_response_tests.rs"]
mod tests;
