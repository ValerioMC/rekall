use rekall_model::{TaskStepState, WrapupAuthor};

use super::*;

fn task_at(claimed_at: Option<Instant>) -> task::Model {
    let mut task = task::Model::new("pay".into(), "Pay".into(), Id::random(), Instant::now());
    task.claimed_at = claimed_at;
    task
}

fn step_of(task: &task::Model, running_at: Instant) -> task_step::Model {
    let mut step = task_step::Model::new(task.id, "Wire the form".into(), 0, running_at);
    step.state = TaskStepState::Running;
    step.running_at = Some(running_at);
    step
}

fn wrapup_of(task: &task::Model, at: Instant) -> wrapup::Model {
    wrapup::Model { id: Id::random(), task_id: task.id, body_markdown: "state".into(), written_by: WrapupAuthor::Claude, created_at: at, updated_at: at }
}

#[test]
fn the_feed_lists_every_recorded_moment_newest_first() {
    let start = Instant::now();
    let task = task_at(Some(start.plus_minutes(3)));
    let steps = [step_of(&task, start.plus_minutes(1))];
    let wrapups = [wrapup_of(&task, start.plus_minutes(2))];

    let feed = EpicActivityResponse::feed(std::slice::from_ref(&task), &steps, &wrapups, &[]);

    let kinds: Vec<EpicActivityKind> = feed.iter().map(|entry| entry.kind).collect();
    assert_eq!(kinds, [EpicActivityKind::TaskClaimed, EpicActivityKind::WrapupWritten, EpicActivityKind::StepStarted]);
    assert_eq!(feed[2].subject.as_deref(), Some("Wire the form"));
}

#[test]
fn a_moment_never_reached_is_left_out() {
    let task = task_at(None);
    assert!(EpicActivityResponse::feed(&[task], &[], &[], &[]).is_empty());
}

#[test]
fn the_feed_stops_at_its_cap() {
    let start = Instant::now();
    let task = task_at(None);
    let steps: Vec<task_step::Model> = (0..ACTIVITY_MAX as i64 + 5).map(|minute| step_of(&task, start.plus_minutes(minute))).collect();

    let feed = EpicActivityResponse::feed(&[task], &steps, &[], &[]);

    assert_eq!(feed.len(), ACTIVITY_MAX);
    assert_eq!(feed[0].at, start.plus_minutes(ACTIVITY_MAX as i64 + 4));
}
