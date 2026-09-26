use super::*;

fn new_step() -> Model {
    Model::new(Id::random(), "Aggregate the rows".into(), 0, Instant::now())
}

#[test]
fn a_step_opens_draft_with_nothing_behind_it() {
    let step = new_step();
    assert_eq!(step.state, TaskStepState::Draft);
    assert!(!step.is_done());
    assert!(step.running_at.is_none());
    assert!(step.claimed_at.is_none());
    assert!(step.done_at.is_none());
    assert!(step.completed_at().is_none());
}

#[test]
fn promoting_a_draft_to_open_leaves_nothing_behind_it() {
    let mut step = new_step();
    step.mark_state(TaskStepState::Open);
    assert_eq!(step.state, TaskStepState::Open);
    assert!(step.running_at.is_none());
    assert!(step.claimed_at.is_none());
    assert!(step.done_at.is_none());
}

#[test]
fn running_then_claimed_then_done_each_move_stamps_its_own_moment() {
    let mut step = new_step();
    step.mark_state(TaskStepState::Running);
    assert!(step.running_at.is_some());
    assert!(step.claimed_at.is_none());
    assert!(step.done_at.is_none());

    step.mark_state(TaskStepState::Claimed);
    assert!(step.claimed_at.is_some());
    assert!(step.done_at.is_none());
    assert_eq!(step.completed_at(), step.claimed_at, "finished when it was claimed, not when ticked");

    step.mark_state(TaskStepState::Done);
    assert!(step.is_done());
    assert!(step.done_at.is_some());
    assert_eq!(step.completed_at(), step.claimed_at, "still measured from the claim");
}

#[test]
fn claiming_straight_from_open_still_records_that_it_must_have_been_running() {
    let mut step = new_step();
    step.mark_state(TaskStepState::Claimed);
    assert!(step.running_at.is_some());
    assert!(step.claimed_at.is_some());
}

#[test]
fn reopening_clears_every_moment() {
    let mut step = new_step();
    step.mark_state(TaskStepState::Running);
    step.mark_state(TaskStepState::Claimed);
    step.mark_state(TaskStepState::Done);

    step.mark_state(TaskStepState::Open);

    assert_eq!(step.state, TaskStepState::Open);
    assert!(step.running_at.is_none());
    assert!(step.claimed_at.is_none());
    assert!(step.done_at.is_none());
    assert!(step.completed_at().is_none());
}

#[test]
fn marking_a_step_the_state_it_is_already_in_changes_nothing() {
    let mut step = new_step();
    step.mark_state(TaskStepState::Running);
    let running_at = step.running_at;
    std::thread::sleep(std::time::Duration::from_millis(2));
    step.mark_state(TaskStepState::Running);
    assert_eq!(step.running_at, running_at);
}
