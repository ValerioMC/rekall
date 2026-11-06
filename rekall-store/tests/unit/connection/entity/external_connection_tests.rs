use super::*;

fn gitlab(base_url: Option<&str>) -> Model {
    Model::new("gitlab".into(), "GitLab".into(), ConnectionKind::Gitlab, base_url.map(str::to_string), Instant::now())
}

fn kubernetes() -> Model {
    Model::new("prod".into(), "Production".into(), ConnectionKind::Kubernetes, None, Instant::now())
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
    Model::new("jira".into(), "Jira".into(), ConnectionKind::Jira, base_url.map(str::to_string), Instant::now())
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

fn jira_behind(proxy_url: &str) -> Model {
    let mut connection = jira(Some("https://example.atlassian.net"));
    connection.proxy_url = Some(proxy_url.into());
    connection
}

#[test]
fn a_jira_connection_takes_an_http_or_https_proxy() {
    assert!(jira_behind("http://proxy.example.com:8080").validate().is_ok());
    assert!(jira_behind("https://proxy.example.com").validate().is_ok());
    assert!(jira_behind("socks5://proxy.example.com:1080").validate().is_err());
    assert!(jira_behind("http://").validate().is_err());
}

#[test]
fn a_proxy_with_a_user_or_password_is_refused_without_echoing_it() {
    let error = jira_behind("http://ada:s3cret@proxy.example.com:8080").validate().unwrap_err();
    assert!(!error.to_string().contains("s3cret"));
}

#[test]
fn only_a_jira_connection_takes_a_proxy() {
    let mut connection = gitlab(Some("https://gitlab.example.com"));
    connection.proxy_url = Some("http://proxy.example.com:8080".into());
    assert!(connection.validate().is_err());
}

#[test]
fn a_github_connection_without_its_server_url_is_refused() {
    let github = Model::new("github".into(), "GitHub".into(), ConnectionKind::Github, None, Instant::now());
    let error = github.validate().unwrap_err();
    assert!(error.to_string().contains("GitHub connection needs the server URL"));
}

fn gitlab_covering(root_path: &str) -> Model {
    let mut connection = gitlab(Some("https://gitlab.example.com"));
    connection.root_path = Some(root_path.into());
    connection
}

#[test]
fn a_gitlab_or_github_connection_may_name_the_root_path_its_credential_covers() {
    assert!(gitlab_covering("acme/platform").validate().is_ok());
    let mut github = Model::new("github".into(), "GitHub".into(), ConnectionKind::Github, Some("https://github.com".into()), Instant::now());
    github.root_path = Some("vmcforge".into());
    assert!(github.validate().is_ok());
}

#[test]
fn a_root_path_is_stored_without_a_slash_at_either_end() {
    assert!(gitlab_covering("/acme").validate().is_err());
    assert!(gitlab_covering("acme/").validate().is_err());
    assert!(gitlab_covering("  ").validate().is_err());
}

#[test]
fn a_kubernetes_or_jira_connection_takes_no_root_path() {
    let mut cluster = kubernetes();
    cluster.root_path = Some("acme".into());
    assert!(cluster.validate().is_err());
    let mut site = jira(Some("https://example.atlassian.net"));
    site.root_path = Some("acme".into());
    assert!(site.validate().is_err());
}

#[test]
fn a_root_path_over_the_cap_is_refused() {
    assert!(gitlab_covering(&"a".repeat(ROOT_PATH_MAX_CHARACTERS + 1)).validate().is_err());
}

fn grafana(base_url: Option<&str>) -> Model {
    Model::new("grafana".into(), "Grafana".into(), ConnectionKind::Grafana, base_url.map(str::to_string), Instant::now())
}

#[test]
fn a_grafana_connection_without_its_instance_url_is_refused() {
    let error = grafana(None).validate().unwrap_err();
    assert!(error.to_string().contains("instance URL"));
    assert!(grafana(Some("https://grafana.example.com")).validate().is_ok());
}

#[test]
fn a_grafana_connection_takes_neither_a_kubeconfig_context_nor_a_proxy_nor_a_root_path() {
    let mut context = grafana(Some("https://grafana.example.com"));
    context.kube_context = Some("prod".into());
    let mut proxy = grafana(Some("https://grafana.example.com"));
    proxy.proxy_url = Some("http://proxy.example.com:8080".into());
    let mut root = grafana(Some("https://grafana.example.com"));
    root.root_path = Some("acme".into());
    assert!(context.validate().is_err());
    assert!(proxy.validate().is_err());
    assert!(root.validate().is_err());
}
