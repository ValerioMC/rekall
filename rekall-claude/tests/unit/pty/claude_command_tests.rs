use super::*;

#[test]
fn only_the_offered_model_and_effort_values_pass() {
    assert_eq!(normalise(Some(" Opus "), &MODEL_ALIASES).as_deref(), Some("opus"));
    assert_eq!(normalise(Some("gpt"), &MODEL_ALIASES), None);
    assert_eq!(normalise(None, &EFFORT_LEVELS), None);
}
