use super::*;

#[test]
fn a_draft_is_out_of_a_sessions_reach_and_so_is_done() {
    assert!(!TaskStepState::Draft.reachable_by_session());
    assert!(TaskStepState::Open.reachable_by_session());
    assert!(TaskStepState::Running.reachable_by_session());
    assert!(TaskStepState::Claimed.reachable_by_session());
    assert!(!TaskStepState::Done.reachable_by_session());
}

#[test]
fn ordinals_follow_declaration_order() {
    assert_eq!(TaskStepState::Draft.ordinal(), 0);
    assert_eq!(TaskStepState::Done.ordinal(), 4);
}
