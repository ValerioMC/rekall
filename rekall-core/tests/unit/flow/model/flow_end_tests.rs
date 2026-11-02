use rekall_kernel::{Id, Instant};
use rekall_store::TaskStepState;

use super::*;

fn step(state: TaskStepState) -> task_step::Model {
    let mut step = task_step::Model::new(Id::random(), "A step".into(), 0, Instant::now());
    step.state = state;
    step
}

fn visit(step: &task_step::Model, open: bool) -> step_visit::Model {
    let mut visit = step_visit::Model::enter(step.task_id, step.id, 1, Instant::now());
    if !open {
        visit.leave(None, None, Instant::now());
    }
    visit
}

#[test]
fn a_flow_that_has_not_run_has_not_ended() {
    assert!(!flow_has_ended(&[step(TaskStepState::Open)], &[]));
}

#[test]
fn a_flow_standing_on_a_step_has_not_ended() {
    let current = step(TaskStepState::Open);
    assert!(!flow_has_ended(&[current.clone()], &[visit(&current, true)]));
}

#[test]
fn a_flow_with_a_running_step_has_not_ended() {
    let running = step(TaskStepState::Running);
    assert!(!flow_has_ended(&[running.clone()], &[visit(&running, false)]));
}

#[test]
fn a_flow_that_left_its_last_step_has_ended_whatever_it_never_entered() {
    let claimed = step(TaskStepState::Claimed);
    let never_needed = step(TaskStepState::Open);
    assert!(flow_has_ended(&[claimed.clone(), never_needed], &[visit(&claimed, false)]));
}

#[test]
fn a_visited_step_that_is_open_again_makes_the_flow_live() {
    let sent_back = step(TaskStepState::Open);
    assert!(!flow_has_ended(&[sent_back.clone()], &[visit(&sent_back, false)]));
}

#[test]
fn the_steps_entered_are_the_ones_with_a_visit() {
    let ran = step(TaskStepState::Claimed);
    let skipped = step(TaskStepState::Open);
    let all = [ran.clone(), skipped];
    let entered = steps_entered(&all, &[visit(&ran, false)]);
    assert_eq!(entered.len(), 1);
    assert_eq!(entered[0].id, ran.id);
}

#[test]
fn a_checklist_is_a_flow_only_when_a_released_step_carries_a_route() {
    let from = step(TaskStepState::Open);
    let link = step_link::Model {
        id: Id::random(),
        task_id: from.task_id,
        from_step_id: from.id,
        to_step_id: None,
        outcome: "passed".into(),
        position: 0,
        created_at: Instant::now(),
        updated_at: Instant::now(),
    };
    assert!(carries_routes(&[link.clone()], &[from.clone()]));
    assert!(!carries_routes(&[link], &[step(TaskStepState::Open)]));
    assert!(!carries_routes(&[], &[from]));
}
