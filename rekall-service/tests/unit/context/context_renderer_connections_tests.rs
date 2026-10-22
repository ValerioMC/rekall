use super::*;

fn connection(kind: ConnectionKind, label: &str, target: Option<&str>, credential_set: bool) -> ConnectionView {
    ConnectionView {
        kind,
        label: label.into(),
        title: format!("The {label}"),
        target: target.map(str::to_string),
        instance_url: (kind == ConnectionKind::Gitlab).then(|| "https://gitlab.example.com".to_string()),
        credential_set,
    }
}

#[test]
fn a_project_without_connections_renders_no_block() {
    assert_eq!(ContextRenderer.render_connections(&[]), "");
}

#[test]
fn a_cluster_is_reached_by_its_label_and_namespace() {
    let rendered = ContextRenderer.render_connections(&[connection(ConnectionKind::Kubernetes, "prod", Some("apps"), true)]);
    assert!(rendered.starts_with("\n<connections count=\"1\">\n"));
    assert!(rendered.contains("- `prod` (The prod): Kubernetes, `kubectl --context prod`, namespace `apps`\n"));
    assert!(rendered.ends_with("</connections>\n"));
}

#[test]
fn a_cluster_without_a_target_uses_its_default_namespace() {
    let rendered = ContextRenderer.render_connections(&[connection(ConnectionKind::Kubernetes, "prod", None, true)]);
    assert!(rendered.contains("`kubectl --context prod` in its default namespace"));
}

#[test]
fn a_gitlab_connection_names_its_instance_project_and_how_to_push_and_call_it() {
    let rendered = ContextRenderer.render_connections(&[connection(ConnectionKind::Gitlab, "gitlab", Some("group/app"), true)]);
    assert!(rendered.contains("GitLab at https://gitlab.example.com, project `group/app`"));
    assert!(rendered.contains("configured credential helper"));
    assert!(rendered.contains("`rekall-gitlab --connection gitlab <METHOD> <path>`"));
}

#[test]
fn a_connection_without_a_credential_tells_the_session_to_report_it() {
    let rendered = ContextRenderer.render_connections(&[connection(ConnectionKind::Kubernetes, "staging", None, false)]);
    assert!(rendered.contains("- `staging` (The staging): no credential is stored for it. Tell the person"));
    assert!(!rendered.contains("kubectl --context staging"));
}

#[test]
fn the_block_forbids_handling_credentials_directly() {
    let rendered = ContextRenderer.render_connections(&[connection(ConnectionKind::Kubernetes, "prod", None, true)]);
    assert!(rendered.contains("Never print, echo or write a credential or a kubeconfig"));
    assert!(rendered.contains("only in a terminal Rekall opened"));
}

#[test]
fn a_jira_connection_names_its_site_project_and_how_to_call_it() {
    let jira = ConnectionView { instance_url: Some("https://example.atlassian.net".into()), ..connection(ConnectionKind::Jira, "jira", Some("ENG"), true) };
    let rendered = ContextRenderer.render_connections(&[jira]);
    assert!(rendered.contains("Jira at https://example.atlassian.net, project `ENG`"));
    assert!(rendered.contains("`rekall-jira --connection jira <METHOD> <path> [curl options]`"));
    assert!(rendered.contains("`POST /issue`"));
}
