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
        proxy_url: None,
        secret_ref: "connection-x".into(),
        credential_set: true,
    }
}

fn jira() -> TerminalConnection {
    TerminalConnection {
        kind: ConnectionKind::Jira,
        target: Some("ENG".into()),
        instance_url: Some("https://example.atlassian.net".into()),
        ..gitlab("jira", None)
    }
}

#[test]
fn every_helper_sends_the_token_on_stdin_never_as_an_argument() {
    for script in [kube_plugin(ENDPOINT), git_helper(ENDPOINT), api_wrapper(&GITLAB_API, ENDPOINT, &[&gitlab("gitlab", None)]), api_wrapper(&JIRA_API, ENDPOINT, &[&jira()])] {
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
    let script = api_wrapper(&GITLAB_API, ENDPOINT, &[&gitlab("gitlab", Some("acme/vega app"))]);
    assert!(script.contains("  'gitlab') base='https://gitlab.example.com/api/v4'; project='acme%2Fvega%20app'; proxy='' ;;"));
    assert!(script.contains("\"$label?for=gitlab\""));
}

#[test]
fn the_gitlab_wrapper_sends_only_the_token_header() {
    let script = api_wrapper(&GITLAB_API, ENDPOINT, &[&gitlab("gitlab", None)]);
    assert!(script.contains("curl --silent --show-error -X \"$method\" -H @- \"$base$path\" \"$@\""));
}

#[test]
fn the_jira_wrapper_calls_the_rest_api_of_each_site_with_json_both_ways() {
    let script = api_wrapper(&JIRA_API, ENDPOINT, &[&jira()]);
    assert!(script.contains("usage: rekall-jira --connection <label> <METHOD> <path> [curl options]"));
    assert!(script.contains("  'jira') base='https://example.atlassian.net/rest/api/3'; project='ENG'; proxy='' ;;"));
    assert!(script.contains("\"$label?for=jira\""));
    assert!(script.contains("-H @- -H 'Accept: application/json' -H 'Content-Type: application/json' \"$base$path\""));
    assert!(script.contains("this terminal has no Jira connection labelled"));
}

#[test]
fn the_jira_wrapper_sends_its_calls_through_the_connections_proxy() {
    let behind = TerminalConnection { proxy_url: Some("http://proxy.example.com:8080".into()), ..jira() };
    let script = api_wrapper(&JIRA_API, ENDPOINT, &[&behind]);
    assert!(script.contains("project='ENG'; proxy='http://proxy.example.com:8080' ;;"));
    assert!(script.contains("[ -n \"$proxy\" ] && set -- --proxy \"$proxy\" \"$@\""));
}

#[test]
fn a_quote_in_a_value_cannot_end_its_shell_word() {
    assert_eq!(quoted("it's"), r"'it'\''s'");
}
