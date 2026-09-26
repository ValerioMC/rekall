use super::*;

#[test]
fn an_ambiguous_anchor_lists_its_candidates() {
    let error = RekallError::ambiguous("setup", vec!["project:a".into(), "project:b".into()]);
    assert_eq!(error.message(), "'setup' matches 2 records: project:a, project:b");
}

#[test]
fn a_unique_violation_carries_the_constraint_name() {
    let cause = integrity_cause("error returned from database: (code: 2067) UNIQUE constraint failed: task.project_id, task.label");
    assert!(cause.unwrap().contains("UQ_TASK_PROJECT_LABEL"));
}
