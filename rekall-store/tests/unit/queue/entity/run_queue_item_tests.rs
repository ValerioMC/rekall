use super::*;

#[test]
fn starting_stamps_once_settling_stamps_and_requeueing_clears() {
    let mut item = Model::new(Id::random(), 0, Instant::now());
    item.move_to(RunQueueItemState::Running, None);
    let started = item.started_at;
    assert!(started.is_some());
    item.move_to(RunQueueItemState::Queued, Some("  paused  "));
    assert_eq!(item.detail.as_deref(), Some("paused"));
    item.move_to(RunQueueItemState::Running, None);
    assert_eq!(item.started_at, started);
    item.move_to(RunQueueItemState::Finished, None);
    assert!(item.finished_at.is_some());
    let long = "x".repeat(600);
    item.move_to(RunQueueItemState::Failed, Some(&long));
    assert_eq!(jstr::len(item.detail.as_deref().unwrap()), DETAIL_MAX);
}
