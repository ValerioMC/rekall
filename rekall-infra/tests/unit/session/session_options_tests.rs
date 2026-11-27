use super::*;

#[test]
fn a_versioned_choice_reaches_claude_code_as_the_provider_spells_it() {
    let options = SessionOptions::chosen(false, Some("claude-example-7[1m]"), Some("High"));

    assert_eq!(options.arguments(), ["--model", "claude-example-7[1m]", "--effort", "high"].map(OsString::from));
}

#[test]
fn an_alias_is_passed_in_lowercase() {
    let options = SessionOptions::chosen(false, Some(" Sonnet "), Some("high"));

    assert_eq!(options.arguments(), ["--model", "sonnet", "--effort", "high"].map(OsString::from));
}

#[test]
fn the_account_default_and_a_malformed_choice_pass_no_override() {
    for (model, effort) in [(None, None), (Some("default"), Some("default")), (Some("--help"), Some("extreme")), (Some(" "), Some(""))] {
        assert!(SessionOptions::chosen(false, model, effort).arguments().is_empty(), "{model:?} {effort:?}");
    }
}

#[test]
fn codex_uses_its_own_flags_and_preserves_model_ids() {
    let options = SessionOptions::codex(true, Some("gpt-6.2-sol"), Some("xhigh")).unwrap();
    assert_eq!(
        options.arguments(),
        ["--no-daemon", "--dangerously-bypass-approvals-and-sandbox", "--model", "gpt-6.2-sol", "-c", "model_reasoning_effort=\"xhigh\""].map(OsString::from)
    );
    assert_eq!(SessionOptions::codex(false, Some("default"), Some("default")).unwrap().arguments(), ["--no-daemon"]);
    assert!(SessionOptions::codex(false, Some("--help"), None).is_err());
    assert!(SessionOptions::codex(false, None, Some("high\"\nmodel=\"other")).is_err());
}
