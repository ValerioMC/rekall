use super::*;

#[test]
fn only_the_offered_model_and_effort_values_pass() {
    assert_eq!(normalise(Some(" Opus "), &MODEL_ALIASES).as_deref(), Some("opus"));
    assert_eq!(normalise(Some("gpt"), &MODEL_ALIASES), None);
    assert_eq!(normalise(None, &EFFORT_LEVELS), None);
}

fn arguments(options: &ClaudeOptions) -> Vec<String> {
    let command = claude_command(Path::new("/bin/claude"), options, "/rk project:vega", HashMap::new(), Path::new("/work"));
    command.get_argv().iter().map(|argument| argument.to_string_lossy().into_owned()).collect()
}

#[test]
fn a_credential_sessions_settings_are_passed_to_claude() {
    let mut options = ClaudeOptions::chosen(false, None, None);
    options.settings = Some(PathBuf::from("/s/t1/claude-settings.json"));
    let argv = arguments(&options);
    let at = argv.iter().position(|argument| argument == "--settings").expect("--settings is passed");
    assert_eq!(argv[at + 1], "/s/t1/claude-settings.json");
}

#[test]
fn without_a_credential_session_no_settings_are_passed() {
    assert!(!arguments(&ClaudeOptions::chosen(true, Some("opus"), None)).contains(&"--settings".to_string()));
}
