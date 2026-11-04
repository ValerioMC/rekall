use rekall_kernel::{Id, Instant};

use super::*;

fn step(title: &str, position: i32, state: TaskStepState) -> task_step::Model {
    let mut step = task_step::Model::new(Id::random(), title.into(), position, Instant::now());
    step.state = state;
    step
}

#[test]
fn steps_are_numbered_from_one_in_checklist_order() {
    let steps = [step("Model", 0, TaskStepState::Done), step("Service", 1, TaskStepState::Open)];

    assert_eq!(step_at(&steps, 1).map(|found| found.title.as_str()), Some("Model"));
    assert_eq!(step_at(&steps, 2).map(|found| found.title.as_str()), Some("Service"));
}

#[test]
fn a_draft_is_not_counted_and_a_number_past_the_checklist_names_no_step() {
    let steps = [step("Model", 0, TaskStepState::Draft), step("Service", 1, TaskStepState::Open)];

    assert_eq!(step_at(&steps, 1).map(|found| found.title.as_str()), Some("Service"));
    assert!(step_at(&steps, 2).is_none());
    assert!(step_at(&steps, 0).is_none());
    assert!(step_at(&steps, -1).is_none());
    assert_eq!(planned_count(&steps), 1);
}
