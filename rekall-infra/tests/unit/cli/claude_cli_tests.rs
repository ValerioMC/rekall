use super::*;

struct Fixed(HashMap<String, String>);

#[async_trait::async_trait]
impl EnvironmentSource for Fixed {
    async fn current(&self) -> HashMap<String, String> {
        self.0.clone()
    }
}

fn cli_with(override_path: &str, environment: &[(&str, &str)]) -> ClaudeCli {
    let map = environment.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    ClaudeCli::new(Some(override_path), PathBuf::from("/home/test-user"), Arc::new(Fixed(map)))
}

#[tokio::test]
async fn the_login_shells_variables_reach_the_terminal_its_path_first() {
    let cli = cli_with("", &[("PATH", "/home/me/.cargo/bin:/usr/bin:/bin"), ("RUSTUP_HOME", "/home/me/.rustup"), ("SSH_AUTH_SOCK", "/tmp/agent.sock")]);
    let environment = cli.environment().await;
    assert_eq!(environment["RUSTUP_HOME"], "/home/me/.rustup");
    assert_eq!(environment["SSH_AUTH_SOCK"], "/tmp/agent.sock");
    assert_eq!(environment["PATH"], "/home/me/.cargo/bin:/usr/bin:/bin:/opt/homebrew/bin:/usr/local/bin");
}

#[tokio::test]
async fn an_enclosing_claude_code_sessions_markers_are_not_passed_on() {
    let cli = cli_with("", &[("PATH", "/usr/bin"), ("CLAUDECODE", "1"), ("CLAUDE_CODE_ENTRYPOINT", "cli")]);
    let environment = cli.environment().await;
    assert!(!environment.contains_key("CLAUDECODE") && !environment.contains_key("CLAUDE_CODE_ENTRYPOINT"));
}

#[tokio::test]
async fn with_no_path_at_all_the_usual_install_directories_stand_in() {
    let environment = cli_with("", &[]).environment().await;
    assert_eq!(environment["PATH"], "/opt/homebrew/bin:/usr/local/bin:/usr/bin");
    assert!(environment.contains_key("HOME"));
}

#[tokio::test]
async fn an_override_that_is_executable_wins_one_that_is_not_finds_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("claude");
    std::fs::write(&binary, "#!/bin/sh\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    assert_eq!(cli_with(&binary.to_string_lossy(), &[]).locate().await, Some(binary.clone()));
    assert_eq!(cli_with(&dir.path().join("missing").to_string_lossy(), &[]).locate().await, None);
}
