use rekall_kernel::{Id, Instant};

use super::*;

fn step(task: Id, title: &str, state: TaskStepState) -> task_step::Model {
    let mut step = task_step::Model::new(task, title.into(), 0, Instant::now());
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

fn route(from: &task_step::Model) -> step_link::Model {
    step_link::Model {
        id: Id::random(),
        task_id: from.task_id,
        from_step_id: from.id,
        to_step_id: None,
        outcome: "passed".into(),
        position: 0,
        created_at: Instant::now(),
        updated_at: Instant::now(),
    }
}

fn states(progress: &[StepProgress]) -> Vec<StepProgressState> {
    progress.iter().map(|step| step.state).collect()
}

fn currents(progress: &[StepProgress]) -> Vec<&str> {
    progress.iter().filter(|step| step.current).map(|step| step.title.as_str()).collect()
}

#[test]
fn a_checklist_marks_the_first_open_step_as_current() {
    let task = Id::random();
    let steps = [step(task, "One", TaskStepState::Claimed), step(task, "Two", TaskStepState::Open), step(task, "Three", TaskStepState::Open)];

    let progress = progress_of(&steps, &[], &[]);

    assert_eq!(states(&progress), vec![StepProgressState::Claimed, StepProgressState::Open, StepProgressState::Open]);
    assert_eq!(currents(&progress), vec!["Two"]);
}

#[test]
fn a_running_step_is_the_current_one() {
    let task = Id::random();
    let steps = [step(task, "One", TaskStepState::Open), step(task, "Two", TaskStepState::Running)];

    assert_eq!(currents(&progress_of(&steps, &[], &[])), vec!["Two"]);
}

#[test]
fn nothing_is_current_once_every_step_is_claimed_or_done() {
    let task = Id::random();
    let steps = [step(task, "One", TaskStepState::Claimed), step(task, "Two", TaskStepState::Done)];

    let progress = progress_of(&steps, &[], &[]);

    assert!(currents(&progress).is_empty());
    assert_eq!(states(&progress), vec![StepProgressState::Claimed, StepProgressState::Done]);
}

#[test]
fn drafts_are_not_listed() {
    let task = Id::random();
    let steps = [step(task, "Real", TaskStepState::Open), step(task, "Idea", TaskStepState::Draft)];

    assert_eq!(progress_of(&steps, &[], &[]).len(), 1);
}

#[test]
fn a_flow_standing_on_a_step_makes_it_current_whatever_its_place() {
    let task = Id::random();
    let steps = [step(task, "Build", TaskStepState::Claimed), step(task, "Fix", TaskStepState::Open), step(task, "Test", TaskStepState::Open)];
    let visits = [visit(&steps[0], false), visit(&steps[2], true)];

    assert_eq!(currents(&progress_of(&steps, &visits, &[route(&steps[0])])), vec!["Test"]);
}

#[test]
fn a_flow_that_ended_early_leaves_its_unentered_steps_not_needed_and_nothing_current() {
    let task = Id::random();
    let steps = [step(task, "Check", TaskStepState::Claimed), step(task, "Fix", TaskStepState::Open), step(task, "Report", TaskStepState::Open)];
    let visits = [visit(&steps[0], false)];

    let progress = progress_of(&steps, &visits, &[route(&steps[0])]);

    assert_eq!(states(&progress), vec![StepProgressState::Claimed, StepProgressState::NotNeeded, StepProgressState::NotNeeded]);
    assert!(currents(&progress).is_empty());
}

#[test]
fn a_flow_not_started_keeps_its_open_steps_open() {
    let task = Id::random();
    let steps = [step(task, "Check", TaskStepState::Open), step(task, "Fix", TaskStepState::Open)];

    let progress = progress_of(&steps, &[], &[route(&steps[0])]);

    assert_eq!(states(&progress), vec![StepProgressState::Open, StepProgressState::Open]);
    assert_eq!(currents(&progress), vec!["Check"]);
}

#[test]
fn a_step_sent_back_on_a_flow_is_open_and_current_again() {
    let task = Id::random();
    let steps = [step(task, "Check", TaskStepState::Open), step(task, "Fix", TaskStepState::Open)];
    let visits = [visit(&steps[0], false)];

    let progress = progress_of(&steps, &visits, &[route(&steps[0])]);

    assert_eq!(states(&progress), vec![StepProgressState::Open, StepProgressState::Open]);
    assert_eq!(currents(&progress), vec!["Check"]);
}
