use super::*;

fn model(value: &str) -> Option<String> {
    claude_model_choice(Some(value)).unwrap()
}

#[test]
fn an_alias_and_an_effort_are_trimmed_and_lowercased() {
    assert_eq!(model(" Opus ").as_deref(), Some("opus"));
    assert_eq!(claude_effort_choice(Some(" XHigh ")).unwrap().as_deref(), Some("xhigh"));
}

#[test]
fn a_provider_value_keeps_its_own_spelling() {
    for value in [
        "claude-example-7[1m]",
        "claude-haiku-4-5-20251001",
        "opus[1m]",
        "us.anthropic.claude-Example-v1:0",
        "claude-example-7@20260101",
    ] {
        assert_eq!(model(value).as_deref(), Some(value));
    }
    assert_eq!(model("  claude-example-7  ").as_deref(), Some("claude-example-7"));
}

#[test]
fn nothing_chosen_and_the_word_default_leave_the_account_default() {
    assert_eq!(claude_model_choice(None).unwrap(), None);
    assert_eq!(claude_effort_choice(None).unwrap(), None);
    assert_eq!(model(" Default "), None);
    assert_eq!(claude_effort_choice(Some("default")).unwrap(), None);
}

#[test]
fn a_value_that_is_not_safe_as_an_argument_is_refused() {
    let too_long = "c".repeat(151);
    for value in [
        "",
        " ",
        "--help",
        "[1m]",
        "claude example",
        "claude\"example",
        "claude-example[2m]",
        "claude-example[1m][1m]",
        too_long.as_str(),
    ] {
        assert!(claude_model_choice(Some(value)).is_err(), "{value:?} was accepted");
    }
    assert_eq!(model(&"c".repeat(150)).map(|accepted| accepted.len()), Some(150));
}

#[test]
fn an_effort_outside_the_known_levels_is_refused() {
    let refusal = claude_effort_choice(Some("extreme")).unwrap_err();

    assert_eq!(refusal, RekallError::illegal("'extreme' is not a effort Claude Code accepts."));
}
