use super::*;

#[test]
fn no_outcome_routes_by_nothing() {
    assert_eq!(outcome_for(true, TaskStepState::Claimed, None).unwrap(), None);
    assert_eq!(outcome_for(false, TaskStepState::Claimed, Some("  ")).unwrap(), None);
}

#[test]
fn an_outcome_on_a_flow_claim_is_normalised() {
    assert_eq!(outcome_for(true, TaskStepState::Claimed, Some("App Error")).unwrap().as_deref(), Some("app-error"));
}

#[test]
fn an_outcome_on_a_plain_checklist_is_refused() {
    let error = outcome_for(false, TaskStepState::Claimed, Some("passed")).unwrap_err().to_string();
    assert!(error.contains("carry no routes"), "{error}");
}

#[test]
fn an_outcome_with_a_start_is_refused() {
    assert!(outcome_for(true, TaskStepState::Running, Some("passed")).is_err());
}
