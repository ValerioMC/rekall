use super::*;

fn open(quoted_markdown: &str) -> Option<OpenRevision<'_>> {
    Some(OpenRevision { quoted_markdown })
}

#[test]
fn a_task_nobody_has_evaluated_records_nothing() {
    assert_eq!(decide(false, None, Some("a"), Some("b")), RevisionDecision::Ignore);
}

#[test]
fn a_task_is_evaluated_by_a_wrapup_a_started_step_or_a_review_line_past_open() {
    let unstarted = [TaskStepState::Draft, TaskStepState::Open];
    assert!(!is_evaluated(false, &unstarted, TaskStepState::Open));
    assert!(is_evaluated(true, &unstarted, TaskStepState::Open));
    assert!(is_evaluated(false, &[TaskStepState::Open, TaskStepState::Running], TaskStepState::Open));
    assert!(is_evaluated(false, &[TaskStepState::Claimed], TaskStepState::Open));
    assert!(is_evaluated(false, &[], TaskStepState::Claimed));
}

#[test]
fn the_first_edit_of_an_evaluated_task_opens_a_revision_quoting_the_text_before() {
    assert_eq!(
        decide(true, None, Some("Export to CSV."), Some("Export to CSV and XLSX.")),
        RevisionDecision::Open { quoted: "Export to CSV.".into(), body: "Export to CSV and XLSX.".into() }
    );
}

#[test]
fn a_description_written_for_the_first_time_after_the_evaluation_is_quoted_as_empty() {
    assert_eq!(decide(true, None, None, Some("Now it says something.")), RevisionDecision::Open { quoted: String::new(), body: "Now it says something.".into() });
}

#[test]
fn a_further_edit_rewrites_the_pending_revision() {
    assert_eq!(decide(true, open("v1"), Some("v2"), Some("v3")), RevisionDecision::Update { body: "v3".into() });
}

#[test]
fn going_back_to_the_quoted_text_removes_the_revision() {
    assert_eq!(decide(true, open("v1"), Some("v2"), Some("v1")), RevisionDecision::Remove);
}

#[test]
fn an_unchanged_or_cleared_description_changes_nothing() {
    assert_eq!(decide(true, None, Some("same"), Some("same")), RevisionDecision::Ignore);
    assert_eq!(decide(true, open("v1"), Some("v2"), None), RevisionDecision::Ignore);
    assert_eq!(decide(true, None, Some("v1"), Some("   ")), RevisionDecision::Ignore);
}

#[test]
fn a_task_with_a_plan_applies_its_revisions_to_the_plan_and_one_without_to_the_work() {
    assert_eq!(default_target(true), FeedbackKind::Plan);
    assert_eq!(default_target(false), FeedbackKind::Implementation);
}
