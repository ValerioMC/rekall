use super::*;

fn session() -> SessionEnvironment {
    SessionEnvironment {
        path_prefix: Some(PathBuf::from("/s/t1")),
        kubeconfig: Some(PathBuf::from("/s/t1/kubeconfig")),
        git_config: vec![("credential.https://gitlab.example.com.helper".into(), String::new()), ("credential.https://gitlab.example.com.helper".into(), "!h gitlab".into())],
        token: Some("abc".into()),
        claude_settings: Some(PathBuf::from("/s/t1/claude-settings.json")),
    }
}

#[test]
fn the_session_folder_goes_first_on_the_path_and_the_rest_is_set() {
    let mut environment = HashMap::from([("PATH".to_string(), "/usr/bin:/bin".to_string())]);
    session().apply(&mut environment);
    assert_eq!(environment["PATH"], "/s/t1:/usr/bin:/bin");
    assert_eq!(environment["KUBECONFIG"], "/s/t1/kubeconfig");
    assert_eq!(environment["REKALL_CREDENTIAL_TOKEN"], "abc");
    assert_eq!(environment["GIT_CONFIG_COUNT"], "2");
    assert_eq!(environment["GIT_CONFIG_VALUE_1"], "!h gitlab");
}

#[test]
fn git_entries_follow_those_the_shell_already_set() {
    let mut environment = HashMap::from([
        ("GIT_CONFIG_COUNT".to_string(), "1".to_string()),
        ("GIT_CONFIG_KEY_0".to_string(), "core.pager".to_string()),
        ("GIT_CONFIG_VALUE_0".to_string(), "less".to_string()),
    ]);
    session().apply(&mut environment);
    assert_eq!(environment["GIT_CONFIG_COUNT"], "3");
    assert_eq!(environment["GIT_CONFIG_KEY_0"], "core.pager");
    assert_eq!(environment["GIT_CONFIG_KEY_1"], "credential.https://gitlab.example.com.helper");
}

#[test]
fn an_empty_session_changes_nothing() {
    let mut environment = HashMap::from([("PATH".to_string(), "/usr/bin".to_string())]);
    SessionEnvironment::default().apply(&mut environment);
    assert_eq!(environment, HashMap::from([("PATH".to_string(), "/usr/bin".to_string())]));
}
