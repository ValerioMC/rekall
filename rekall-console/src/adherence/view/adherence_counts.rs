use rekall_kernel::Result;
use rekall_store::{task_step, TaskStepState};

/// The step numbers of a plan. `passes_total` counts each started step once, plus once more for
/// every time it was sent back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AdherenceCounts {
    pub steps_total: usize,
    pub steps_done: usize,
    pub steps_claimed: usize,
    pub accepted_first_time: usize,
    pub sent_back: usize,
    pub passes_total: usize,
}

impl AdherenceCounts {
    pub fn of(steps: &[task_step::Model]) -> Result<Self> {
        let mut counts = Self { steps_total: steps.len(), ..Self::default() };
        for step in steps {
            let sent_back_times = step.passes()?.len();
            counts.steps_done += usize::from(step.state == TaskStepState::Done);
            counts.steps_claimed += usize::from(step.state == TaskStepState::Claimed);
            counts.accepted_first_time += usize::from(step.state == TaskStepState::Done && sent_back_times == 0);
            counts.sent_back += usize::from(sent_back_times > 0);
            counts.passes_total += sent_back_times + usize::from(has_started(step));
        }
        Ok(counts)
    }

    pub fn plus(self, other: Self) -> Self {
        Self {
            steps_total: self.steps_total + other.steps_total,
            steps_done: self.steps_done + other.steps_done,
            steps_claimed: self.steps_claimed + other.steps_claimed,
            accepted_first_time: self.accepted_first_time + other.accepted_first_time,
            sent_back: self.sent_back + other.sent_back,
            passes_total: self.passes_total + other.passes_total,
        }
    }

    /// Accepted at the first pass over accepted; `None` before any step is accepted, which is not zero.
    pub fn first_time_share(&self) -> Option<f64> {
        (self.steps_done > 0).then(|| self.accepted_first_time as f64 / self.steps_done as f64)
    }
}

fn has_started(step: &task_step::Model) -> bool {
    matches!(step.state, TaskStepState::Running | TaskStepState::Claimed | TaskStepState::Done)
}
