use super::*;

#[test]
fn only_task_work_launches_follow_a_step() {
    let task_id = Id::random();

    assert_eq!(guard_scope(Some(task_id), &TerminalMode::Work), GuardScope::Work(task_id));
    assert_eq!(guard_scope(Some(task_id), &TerminalMode::Refine), GuardScope::WithoutStep);
    assert_eq!(guard_scope(None, &TerminalMode::Work), GuardScope::WithoutStep);
}
