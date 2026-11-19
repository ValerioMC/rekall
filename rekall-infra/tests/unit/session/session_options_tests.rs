use super::*;

#[test]
fn only_the_offered_model_and_effort_values_pass() {
    assert_eq!(normalise(Some(" Opus "), &MODEL_ALIASES).as_deref(), Some("opus"));
    assert_eq!(normalise(Some("gpt"), &MODEL_ALIASES), None);
    assert_eq!(normalise(None, &EFFORT_LEVELS), None);
}

#[test]
fn codex_uses_its_own_flags_and_preserves_model_ids() {
    let options = SessionOptions::codex(true, Some("gpt-6.2-sol"), Some("xhigh")).unwrap();
    assert_eq!(
        options.arguments(),
        ["--dangerously-bypass-approvals-and-sandbox", "--model", "gpt-6.2-sol", "-c", "model_reasoning_effort=\"xhigh\""].map(OsString::from)
    );
    assert!(SessionOptions::codex(false, Some("default"), Some("default")).unwrap().arguments().is_empty());
    assert!(SessionOptions::codex(false, Some("--help"), None).is_err());
    assert!(SessionOptions::codex(false, None, Some("high\"\nmodel=\"other")).is_err());
}
