use std::path::PathBuf;

use super::*;

fn arguments(options: &ClaudeOptions) -> Vec<String> {
    let command = chat_command(Path::new("/bin/claude"), options, HashMap::new(), Path::new("/work"));
    command.as_std().get_args().map(|argument| argument.to_string_lossy().into_owned()).collect()
}

fn value_after(argv: &[String], flag: &str) -> Option<String> {
    let at = argv.iter().position(|argument| argument == flag)?;
    argv.get(at + 1).cloned()
}

#[test]
fn it_speaks_stream_json_on_both_pipes_with_partial_messages() {
    let argv = arguments(&ClaudeOptions::chosen(true, None, None));
    assert_eq!(&argv[..7], HEADLESS_ARGUMENTS);
}

#[test]
fn with_skip_permissions_claude_is_never_asked_to_prompt() {
    let argv = arguments(&ClaudeOptions::chosen(true, None, None));
    assert!(argv.contains(&"--dangerously-skip-permissions".to_string()));
    assert!(!argv.contains(&"--permission-prompt-tool".to_string()));
}

#[test]
fn without_skip_permissions_approvals_come_back_over_stdio() {
    let argv = arguments(&ClaudeOptions::chosen(false, None, None));
    assert!(!argv.contains(&"--dangerously-skip-permissions".to_string()));
    assert_eq!(value_after(&argv, "--permission-prompt-tool").as_deref(), Some("stdio"));
}

#[test]
fn the_offered_model_and_effort_are_passed() {
    let argv = arguments(&ClaudeOptions::chosen(true, Some("Opus"), Some("high")));
    assert_eq!(value_after(&argv, "--model").as_deref(), Some("opus"));
    assert_eq!(value_after(&argv, "--effort").as_deref(), Some("high"));
}

#[test]
fn a_credential_sessions_settings_are_passed_to_claude() {
    let mut options = ClaudeOptions::chosen(false, None, None);
    options.settings = Some(PathBuf::from("/s/t1/claude-settings.json"));
    assert_eq!(value_after(&arguments(&options), "--settings").as_deref(), Some("/s/t1/claude-settings.json"));
}

#[test]
fn the_process_gets_the_given_environment_and_no_terminal_variables() {
    let environment = HashMap::from([("HOME".to_string(), "/home/person".to_string())]);
    let command = chat_command(Path::new("/bin/claude"), &ClaudeOptions::default(), environment, Path::new("/work"));
    let variables: Vec<String> = command.as_std().get_envs().map(|(key, _)| key.to_string_lossy().into_owned()).collect();
    assert_eq!(variables, vec!["HOME".to_string()]);
    assert_eq!(command.as_std().get_current_dir(), Some(Path::new("/work")));
}
