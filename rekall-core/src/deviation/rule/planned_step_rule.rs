use rekall_store::{task_step, TaskStepState};

/// The step a deviation's number names: steps are numbered from 1 in the order the checklist
/// lists them, and a draft is not on the checklist yet.
pub fn step_at(steps: &[task_step::Model], step_number: i32) -> Option<&task_step::Model> {
    let index = usize::try_from(step_number).ok()?.checked_sub(1)?;
    steps.iter().filter(|step| step.state != TaskStepState::Draft).nth(index)
}

pub fn planned_count(steps: &[task_step::Model]) -> usize {
    steps.iter().filter(|step| step.state != TaskStepState::Draft).count()
}

#[cfg(test)]
#[path = "../../../tests/unit/deviation/rule/planned_step_rule_tests.rs"]
mod tests;
