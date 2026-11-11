use super::*;

#[test]
fn ordinals_follow_declaration_order() {
    assert_eq!(TaskStepState::Draft.ordinal(), 0);
    assert_eq!(TaskStepState::Done.ordinal(), 4);
}
