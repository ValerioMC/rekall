use super::*;

fn gitlab(base_url: Option<&str>) -> Model {
    Model::new(Id::random(), "gitlab".into(), "GitLab".into(), ConnectionKind::Gitlab, base_url.map(str::to_string), Instant::now())
}

fn kubernetes() -> Model {
    Model::new(Id::random(), "prod".into(), "Production".into(), ConnectionKind::Kubernetes, None, Instant::now())
}

#[test]
fn a_gitlab_connection_over_https_is_valid() {
    assert!(gitlab(Some("https://gitlab.example.com")).validate().is_ok());
}

#[test]
fn a_gitlab_connection_without_its_instance_url_is_refused() {
    let error = gitlab(None).validate().unwrap_err();
    assert!(error.to_string().contains("instance URL"));
}

#[test]
fn a_kubernetes_connection_needs_no_url_before_its_kubeconfig_is_imported() {
    assert!(kubernetes().validate().is_ok());
}

#[test]
fn plain_http_is_refused_unless_it_points_at_this_machine() {
    assert!(gitlab(Some("http://gitlab.example.com")).validate().is_err());
    assert!(gitlab(Some("http://localhost:8080")).validate().is_ok());
    assert!(gitlab(Some("http://127.0.0.1/gitlab")).validate().is_ok());
    assert!(gitlab(Some("http://[::1]:8080")).validate().is_ok());
}

#[test]
fn a_loopback_name_hidden_in_a_longer_host_does_not_pass_as_this_machine() {
    assert!(gitlab(Some("http://localhost.example.com")).validate().is_err());
}

#[test]
fn an_https_address_without_a_host_is_refused() {
    assert!(gitlab(Some("https://")).validate().is_err());
}

#[test]
fn a_kubeconfig_context_on_a_gitlab_connection_is_refused() {
    let mut connection = gitlab(Some("https://gitlab.example.com"));
    connection.kube_context = Some("prod".into());
    assert!(connection.validate().is_err());
}

#[test]
fn the_label_has_to_be_a_slug() {
    let mut connection = kubernetes();
    connection.label = "Prod Cluster".into();
    assert!(connection.validate().is_err());
}

#[test]
fn the_keychain_account_is_generated_from_the_id_and_no_credential_is_stored_yet() {
    let connection = kubernetes();
    assert_eq!(connection.secret_ref, format!("connection-{}", connection.id));
    assert!(!connection.has_credential());
}

fn jira(base_url: Option<&str>) -> Model {
    Model::new(Id::random(), "jira".into(), "Jira".into(), ConnectionKind::Jira, base_url.map(str::to_string), Instant::now())
}

#[test]
fn a_jira_connection_without_its_site_url_is_refused() {
    let error = jira(None).validate().unwrap_err();
    assert!(error.to_string().contains("site URL"));
    assert!(jira(Some("https://example.atlassian.net")).validate().is_ok());
}

#[test]
fn a_kubeconfig_context_on_a_jira_connection_is_refused() {
    let mut connection = jira(Some("https://example.atlassian.net"));
    connection.kube_context = Some("prod".into());
    assert!(connection.validate().is_err());
}
