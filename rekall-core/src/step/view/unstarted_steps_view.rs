use std::collections::HashMap;

use rekall_kernel::Id;

use crate::step::TaskStepView;

/// A task's draft and open steps in checklist order, with the number of every step of the task,
/// since a route of theirs may lead to one that is finished.
#[derive(Clone, Debug)]
pub struct UnstartedStepsView {
    pub task_anchor: String,
    pub steps: Vec<TaskStepView>,
    numbers: HashMap<Id, i32>,
}

impl UnstartedStepsView {
    pub fn of(task_anchor: String, checklist: Vec<TaskStepView>) -> Self {
        let numbers = checklist.iter().map(|step| (step.id, step.position + 1)).collect();
        let steps = checklist.into_iter().filter(|step| step.state.unstarted()).collect();
        Self { task_anchor, steps, numbers }
    }

    /// The step's one-based place in the whole checklist, as `rekall_step` takes it.
    pub fn number_of(&self, step_id: Id) -> Option<i32> {
        self.numbers.get(&step_id).copied()
    }
}
