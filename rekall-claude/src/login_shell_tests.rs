use super::*;

fn fake_shell(dir: &std::path::Path, body: &str) -> String {
    let shell = dir.join("fake-shell");
    std::fs::write(&shell, format!("#!/bin/sh\n{body}\neval \"$4\"\n")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    shell.to_string_lossy().into_owned()
}

fn resolve(shell: &str, timeout: u64) -> Option<Environment> {
    LoginShellEnvironment::new(Some(shell), Duration::from_secs(timeout)).resolve()
}

#[test]
fn what_the_profile_exports_comes_back_and_what_it_prints_around_the_markers_does_not() {
    let dir = tempfile::tempdir().unwrap();
    let shell = fake_shell(dir.path(), "export PATH=\"/home/me/.cargo/bin:$PATH\"\nexport RUSTUP_HOME=/home/me/.rustup\necho 'Welcome back, last login yesterday'");
    let environment = resolve(&shell, 5).unwrap();
    assert!(environment["PATH"].starts_with("/home/me/.cargo/bin:"));
    assert_eq!(environment["RUSTUP_HOME"], "/home/me/.rustup");
    assert!(!environment.keys().any(|k| k.contains("Welcome")));
}

#[test]
fn the_resolving_shells_own_variables_and_the_flag_are_left_out() {
    let dir = tempfile::tempdir().unwrap();
    let shell = fake_shell(dir.path(), "[ -n \"$REKALL_RESOLVING_ENVIRONMENT\" ] && export SAW_FLAG=yes");
    let environment = resolve(&shell, 5).unwrap();
    assert_eq!(environment["SAW_FLAG"], "yes");
    for key in ["_", "SHLVL", "PWD", "OLDPWD", RESOLVING_FLAG] {
        assert!(!environment.contains_key(key), "{key}");
    }
}

#[test]
fn a_value_holding_an_equals_sign_or_a_newline_survives_intact() {
    let dir = tempfile::tempdir().unwrap();
    let shell = fake_shell(dir.path(), "export ODD='a=b\nsecond line'");
    assert_eq!(resolve(&shell, 5).unwrap()["ODD"], "a=b\nsecond line");
}

#[test]
fn a_shell_that_never_finishes_is_abandoned_at_the_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let shell = fake_shell(dir.path(), "sleep 30");
    let started = Instant::now();
    assert!(resolve(&shell, 1).is_none());
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[test]
fn a_shell_that_exits_before_printing_or_is_not_there_gives_nothing() {
    let dir = tempfile::tempdir().unwrap();
    assert!(resolve(&fake_shell(dir.path(), "exit 1"), 5).is_none());
    assert!(resolve(&dir.path().join("missing").to_string_lossy(), 5).is_none());
}

#[tokio::test]
async fn with_no_answer_current_falls_back_to_this_processs_environment() {
    let dir = tempfile::tempdir().unwrap();
    let login = LoginShellEnvironment::new(Some(&fake_shell(dir.path(), "exit 1")), Duration::from_secs(5));
    assert_eq!(login.current().await, process_environment());
}

#[tokio::test]
async fn current_hands_back_the_resolved_environment() {
    let dir = tempfile::tempdir().unwrap();
    let login = LoginShellEnvironment::new(Some(&fake_shell(dir.path(), "export FROM_PROFILE=1")), Duration::from_secs(5));
    assert_eq!(login.current().await["FROM_PROFILE"], "1");
}

#[test]
fn parse_needs_both_markers() {
    assert!(parse("noise only", "MARK").is_none());
    assert!(parse("MARKA=1\0", "MARK").is_none());
    let parsed = parse("noise MARKA=1\0B=2\0MARK noise", "MARK").unwrap();
    assert_eq!(parsed, HashMap::from([("A".to_string(), "1".to_string()), ("B".to_string(), "2".to_string())]));
}
