use super::*;
use rekall_kernel::{Id, Instant};
use rekall_store::TaskStepState;

fn steps(states: &[TaskStepState]) -> Vec<task_step::Model> {
    let task_id = Id::random();
    states
        .iter()
        .zip(0..)
        .map(|(state, position)| {
            let mut step = task_step::Model::new(task_id, format!("Step {position}"), position, Instant::now());
            step.state = *state;
            step
        })
        .collect()
}

#[test]
fn a_key_is_a_lowercase_slug_trimmed() {
    assert_eq!(key("  db-password ").unwrap(), "db-password");
    for refused in ["DB password", "", "-lead", &"k".repeat(101)] {
        assert!(key(refused).unwrap_err().message().contains("lowercase slug"), "{refused}");
    }
}

#[test]
fn no_step_means_the_task_as_a_whole() {
    assert_eq!(step_scope(&steps(&[]), None).unwrap(), None);
}

#[test]
fn a_step_must_be_on_the_checklist_and_a_draft_is_not() {
    let checklist = steps(&[TaskStepState::Done, TaskStepState::Open, TaskStepState::Draft]);
    assert_eq!(step_scope(&checklist, Some(2)).unwrap(), Some(2));
    for number in [0, 3] {
        let refused = step_scope(&checklist, Some(number)).unwrap_err();
        assert!(refused.message().contains("checklist has 2 steps: there is no step"), "{refused:?}");
    }
}

#[test]
fn a_task_without_steps_accepts_no_step_number() {
    let refused = step_scope(&steps(&[TaskStepState::Draft]), Some(1)).unwrap_err();
    assert!(refused.message().contains("has no checklist"), "{refused:?}");
}

#[test]
fn a_call_holds_at_most_six_options_and_twelve_lines() {
    assert!(within_call(6, 12).is_ok());
    assert!(within_call(7, 0).unwrap_err().message().contains("at most 6 options"));
    assert!(within_call(0, 13).unwrap_err().message().contains("at most 12 conversation lines"));
}
