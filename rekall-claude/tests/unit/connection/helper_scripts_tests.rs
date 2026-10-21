use rekall_common::Id;
use rekall_model::ConnectionKind;

use super::*;

const ENDPOINT: &str = "http://127.0.0.1:47355/api/terminal/t1/credentials";

fn gitlab(label: &str, target: Option<&str>) -> TerminalConnection {
    TerminalConnection {
        connection_id: Id::random(),
        kind: ConnectionKind::Gitlab,
        label: label.into(),
        target: target.map(str::to_string),
        instance_url: Some("https://gitlab.example.com/".into()),
        secret_ref: "connection-x".into(),
        credential_set: true,
    }
}

#[test]
fn every_helper_sends_the_token_on_stdin_never_as_an_argument() {
    for script in [kube_plugin(ENDPOINT), git_helper(ENDPOINT), gitlab_wrapper(ENDPOINT, &[&gitlab("gitlab", None)])] {
        assert!(script.starts_with("#!/bin/sh\n"));
        assert!(script.contains("printf 'X-Rekall-Credential-Token: %s\\n' \"$REKALL_CREDENTIAL_TOKEN\" |"));
        assert!(script.contains("-H @-"));
    }
}

#[test]
fn the_kube_plugin_asks_for_its_label_as_kubectl() {
    assert!(kube_plugin(ENDPOINT).contains("rekall_fetch 'http://127.0.0.1:47355/api/terminal/t1/credentials/'\"$1?for=kubectl\""));
}

#[test]
fn the_git_helper_answers_get_only() {
    let script = git_helper(ENDPOINT);
    assert!(script.contains("[ \"$2\" = get ] || exit 0"));
    assert!(script.contains("\"$1?for=git\""));
}

#[test]
fn the_gitlab_wrapper_knows_each_instance_and_project_path() {
    let script = gitlab_wrapper(ENDPOINT, &[&gitlab("gitlab", Some("acme/vega app"))]);
    assert!(script.contains("  'gitlab') base='https://gitlab.example.com/api/v4'; project='acme%2Fvega%20app' ;;"));
    assert!(script.contains("\"$label?for=gitlab\""));
}

#[test]
fn a_quote_in_a_value_cannot_end_its_shell_word() {
    assert_eq!(quoted("it's"), r"'it'\''s'");
}
