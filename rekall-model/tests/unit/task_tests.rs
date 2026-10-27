use super::*;

fn new_task() -> Model {
    Model::new("in-app-claude".into(), "In-app Claude execution".into(), Id::random(), Instant::now())
}

#[test]
fn a_fresh_task_opens_open_review_active_with_nothing_behind_it() {
    let task = new_task();
    assert_eq!(task.review_state, TaskStepState::Open);
    assert!(review_active(&[]));
    assert!(task.claimed_at.is_none());
    assert!(task.accepted_at.is_none());
    assert!(task.review_note.is_none());
}

#[test]
fn running_then_claimed_then_accepted_each_move_stamps_its_own_moment() {
    let mut task = new_task();
    task.mark_review_state(TaskStepState::Running);
    assert!(task.claimed_at.is_none());
    assert!(task.accepted_at.is_none());

    task.mark_review_state(TaskStepState::Claimed);
    assert!(task.claimed_at.is_some());
    assert!(task.accepted_at.is_none());

    task.mark_review_state(TaskStepState::Done);
    assert!(task.accepted_at.is_some());
    assert!(task.claimed_at.is_some(), "the claim moment is kept when the work is later accepted");
}

#[test]
fn accepting_straight_from_open_still_records_that_it_must_have_been_claimed() {
    let mut task = new_task();
    task.mark_review_state(TaskStepState::Done);
    assert!(task.claimed_at.is_some());
    assert!(task.accepted_at.is_some());
}

#[test]
fn a_send_back_note_is_kept_while_open_and_dropped_the_moment_it_is_claimed_again() {
    let mut task = new_task();
    task.mark_review_state(TaskStepState::Open);
    task.review_note = Some("the export column is still wrong".into());
    assert_eq!(task.review_note.as_deref(), Some("the export column is still wrong"));

    task.mark_review_state(TaskStepState::Claimed);
    assert!(task.review_note.is_none());
}

#[test]
fn sending_back_clears_every_moment() {
    let mut task = new_task();
    task.mark_review_state(TaskStepState::Running);
    task.mark_review_state(TaskStepState::Claimed);
    task.mark_review_state(TaskStepState::Done);

    task.mark_review_state(TaskStepState::Open);

    assert_eq!(task.review_state, TaskStepState::Open);
    assert!(task.claimed_at.is_none());
    assert!(task.accepted_at.is_none());
}

#[test]
fn marking_the_state_it_is_already_in_changes_nothing() {
    let mut task = new_task();
    task.mark_review_state(TaskStepState::Claimed);
    let claimed_at = task.claimed_at;
    std::thread::sleep(std::time::Duration::from_millis(2));
    task.mark_review_state(TaskStepState::Claimed);
    assert_eq!(task.claimed_at, claimed_at);
}

#[test]
fn only_drafts_count_as_no_checklist() {
    assert!(review_active(&[TaskStepState::Draft, TaskStepState::Draft]));
    assert!(!review_active(&[TaskStepState::Draft, TaskStepState::Open]));
}

fn new_epic(project_id: Id) -> Model {
    Model { kind: TaskKind::Epic, ..Model::new("checkout".into(), "Checkout".into(), project_id, Instant::now()) }
}

#[test]
fn a_task_joins_an_epic_of_its_own_project() {
    let task = new_task();
    assert!(task.require_joinable(&new_epic(task.project_id)).is_ok());
}

#[test]
fn a_task_cannot_join_a_plain_task() {
    let task = new_task();
    let plain = Model::new("other".into(), "Other".into(), task.project_id, Instant::now());
    assert!(task.require_joinable(&plain).is_err());
}

#[test]
fn an_epic_cannot_join_another_epic() {
    let epic = new_epic(Id::random());
    assert!(epic.require_joinable(&new_epic(epic.project_id)).is_err());
}

#[test]
fn a_task_cannot_join_an_epic_of_another_project() {
    let task = new_task();
    assert!(task.require_joinable(&new_epic(Id::random())).is_err());
}

#[test]
fn a_task_cannot_be_its_own_epic() {
    let mut task = new_task();
    let epic = Model { kind: TaskKind::Epic, ..task.clone() };
    task.kind = TaskKind::Task;
    assert!(task.require_joinable(&epic).is_err());
}
