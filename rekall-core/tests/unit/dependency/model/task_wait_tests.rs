use super::*;

fn wait(status: TaskStatus, remaining: Remaining) -> TaskWait {
    TaskWait { task_id: Id::random(), anchor: "task:export".into(), title: "Export".into(), status, remaining }
}

#[test]
fn accepted_work_is_met() {
    let accepted = wait(TaskStatus::InProgress, Remaining::Accepted);

    assert!(accepted.is_met());
    assert_eq!(accepted.state_text(), "accepted");
}

#[test]
fn a_task_set_to_done_is_met() {
    assert!(wait(TaskStatus::Done, Remaining::Open).is_met());
}

#[test]
fn claimed_work_is_not_met_and_says_it_waits_for_review() {
    let claimed = wait(TaskStatus::InProgress, Remaining::Claimed);

    assert!(!claimed.is_met());
    assert_eq!(claimed.state_text(), "claimed, waiting for your review");
}

#[test]
fn open_work_is_not_met() {
    let open = wait(TaskStatus::Todo, Remaining::Open);

    assert!(!open.is_met());
    assert_eq!(open.state_text(), "not accepted");
}
