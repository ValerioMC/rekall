//! A company's connections through the console API: created, given a credential that never comes
//! back, bound to its projects and deleted with their credential.

mod support;

use std::sync::Arc;

use async_trait::async_trait;
use rekall_app::server::StartOptions;
use rekall_service::connection::{InMemorySecretStore, JiraCredential, JiraFailure, JiraIssue, JiraIssues, JiraSite, SecretStore};
use serde_json::json;
use support::{app, app_with, app_with_secrets, id};

const KUBECONFIG: &str = "
clusters:
- name: c
  cluster: {server: 'https://k8s.example.com:6443', certificate-authority-data: Q0E=}
contexts:
- name: prod
  context: {cluster: c, user: u, namespace: apps}
- name: cloud
  context: {cluster: c, user: cloud}
users:
- name: u
  user: {token: sa-token-123}
- name: cloud
  user:
    exec: {command: aws}
";

#[tokio::test]
async fn a_gitlab_connection_keeps_its_token_out_of_every_answer() {
    let secrets = Arc::new(InMemorySecretStore::new());
    let app = app_with_secrets(secrets.clone()).await;
    let acme = app.a_company("Acme").await;

    let created = app.post(&format!("/api/companies/{acme}/connections"), json!({ "kind": "GITLAB", "title": "GitLab", "baseUrl": "https://gitlab.example.com/" })).await;
    assert_eq!((created.status, created.str("label"), created.str("baseUrl")), (201, "gitlab".into(), "https://gitlab.example.com".into()));
    let gitlab = id(&created);

    let set = app.put(&format!("/api/connections/{gitlab}/credential"), json!({ "token": "glpat-secret" })).await;
    assert_eq!(set.status, 200, "{}", set.text);
    assert!(set.get("credentialSetAt").is_string());

    for answer in [set, app.get(&format!("/api/connections/{gitlab}")).await, app.get(&format!("/api/companies/{acme}/connections")).await] {
        assert!(!answer.text.contains("glpat-secret"), "a credential came back: {}", answer.text);
    }
    let account = format!("connection-{gitlab}");
    assert_eq!(secrets.read(&account).unwrap().expose(), "glpat-secret");

    let cleared = app.delete(&format!("/api/connections/{gitlab}/credential")).await;
    assert!(cleared.get("credentialSetAt").is_null());
    assert!(!secrets.contains(&account));
}

#[tokio::test]
async fn a_kubeconfig_is_previewed_without_being_stored_then_imported_for_one_context() {
    let secrets = Arc::new(InMemorySecretStore::new());
    let app = app_with_secrets(secrets.clone()).await;
    let acme = app.a_company("Acme").await;
    let cluster = id(&app.post(&format!("/api/companies/{acme}/connections"), json!({ "kind": "KUBERNETES", "title": "Prod" })).await);

    let preview = app.post("/api/connections/kubeconfig/contexts", json!({ "kubeconfig": KUBECONFIG })).await;
    assert_eq!(preview.status, 200);
    let contexts = preview.list();
    assert_eq!((contexts[0]["name"].as_str(), contexts[0]["supported"].as_bool()), (Some("prod"), Some(true)));
    assert_eq!((contexts[1]["userKind"].as_str(), contexts[1]["supported"].as_bool()), (Some("exec"), Some(false)));
    assert!(!preview.text.contains("sa-token-123"));
    assert!(!secrets.contains(&format!("connection-{cluster}")), "a preview stores nothing");

    let refused = app.put(&format!("/api/connections/{cluster}/credential"), json!({ "kubeconfig": KUBECONFIG, "context": "cloud" })).await;
    assert_eq!(refused.status, 400);
    assert!(refused.detail().contains("runs a command"));

    let imported = app.put(&format!("/api/connections/{cluster}/credential"), json!({ "kubeconfig": KUBECONFIG, "context": "prod" })).await;
    assert_eq!((imported.status, imported.str("baseUrl"), imported.str("kubeContext")), (200, "https://k8s.example.com:6443".into(), "prod".into()));
    assert!(!imported.text.contains("sa-token-123"));
    assert!(secrets.read(&format!("connection-{cluster}")).unwrap().expose().contains("sa-token-123"));
}

#[tokio::test]
async fn a_project_binds_only_its_own_companys_connections_and_sets_a_target() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let globex = app.a_company("Globex").await;
    let vega = app.a_project(&acme, "vega", "ACTIVE").await;
    let theirs = app.a_project(&globex, "orion", "ACTIVE").await;
    let cluster = id(&app.post(&format!("/api/companies/{acme}/connections"), json!({ "kind": "KUBERNETES", "title": "Prod" })).await);

    let bound = app.put(&format!("/api/projects/{vega}/connections/{cluster}"), json!({ "target": " apps " })).await;
    assert_eq!((bound.status, bound.str("target"), bound.str("label")), (200, "apps".into(), "prod".into()));
    let retargeted = app.put(&format!("/api/projects/{vega}/connections/{cluster}"), json!({ "target": "billing" })).await;
    assert_eq!(retargeted.str("target"), "billing", "binding again sets the target");
    assert_eq!(app.get(&format!("/api/projects/{vega}/connections")).await.list().len(), 1);
    assert_eq!(app.get(&format!("/api/connections/{cluster}")).await.get("projects")[0]["label"], "vega");

    assert_eq!(app.put(&format!("/api/projects/{theirs}/connections/{cluster}"), json!({})).await.status, 400);

    assert_eq!(app.delete(&format!("/api/projects/{vega}/connections/{cluster}")).await.status, 204);
    assert_eq!(app.delete(&format!("/api/projects/{vega}/connections/{cluster}")).await.status, 404);
}

#[tokio::test]
async fn deleting_a_connection_unbinds_its_projects_and_forgets_its_credential() {
    let secrets = Arc::new(InMemorySecretStore::new());
    let app = app_with_secrets(secrets.clone()).await;
    let acme = app.a_company("Acme").await;
    let vega = app.a_project(&acme, "vega", "ACTIVE").await;
    let gitlab = id(&app.post(&format!("/api/companies/{acme}/connections"), json!({ "kind": "GITLAB", "title": "GitLab", "baseUrl": "https://gitlab.example.com" })).await);
    app.put(&format!("/api/connections/{gitlab}/credential"), json!({ "token": "glpat-secret" })).await;
    app.put(&format!("/api/projects/{vega}/connections/{gitlab}"), json!({})).await;

    let removed = app.delete(&format!("/api/connections/{gitlab}")).await;

    assert_eq!((removed.status, removed.get("unboundProjects").as_u64()), (200, Some(1)));
    assert!(!secrets.contains(&format!("connection-{gitlab}")));
    assert!(app.get(&format!("/api/projects/{vega}/connections")).await.list().is_empty());
    assert_eq!(app.get(&format!("/api/connections/{gitlab}")).await.status, 404);
}

#[tokio::test]
async fn labels_kinds_and_urls_are_checked() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let url = format!("/api/companies/{acme}/connections");
    let first = id(&app.post(&url, json!({ "kind": "KUBERNETES", "title": "Prod" })).await);

    assert_eq!(app.post(&url, json!({ "kind": "KUBERNETES", "title": "PROD" })).await.status, 409);
    assert_eq!(app.post(&url, json!({ "kind": "SSH", "title": "Box" })).await.status, 400);
    assert_eq!(app.post(&url, json!({ "kind": "GITLAB", "title": "Plain", "baseUrl": "http://gitlab.example.com" })).await.status, 400);
    assert_eq!(app.post(&url, json!({ "kind": "GITLAB", "title": "No URL" })).await.status, 400);
    assert_eq!(app.post(&url, json!({ "kind": "KUBERNETES", "title": "Typed", "baseUrl": "https://k8s.example.com" })).await.status, 400);
    assert_eq!(app.put(&format!("/api/connections/{first}"), json!({ "kind": "GITLAB" })).await.status, 400);
    assert_eq!(app.put(&format!("/api/connections/{first}"), json!({ "title": "Production" })).await.str("title"), "Production");
    assert!(app.get(&format!("/api/connections/{first}/access")).await.list().is_empty());
}

#[tokio::test]
async fn a_check_on_a_connection_without_a_credential_says_so_without_asking_anyone() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let gitlab = id(&app.post(&format!("/api/companies/{acme}/connections"), json!({ "kind": "GITLAB", "title": "GitLab", "baseUrl": "https://gitlab.example.com" })).await);

    let check = app.post_empty(&format!("/api/connections/{gitlab}/check")).await;

    assert_eq!((check.status, check.get("ok").as_bool()), (200, Some(false)), "{}", check.text);
    assert!(check.str("detail").contains("No credential"));
    assert_eq!(app.post_empty("/api/connections/00000000-0000-0000-0000-000000000000/check").await.status, 404);
}

#[tokio::test]
async fn the_context_names_a_projects_connections_and_never_their_credential_or_config() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let vega = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&vega, "deploy").await;
    let cluster = id(&app.post(&format!("/api/companies/{acme}/connections"), json!({ "kind": "KUBERNETES", "title": "Production" })).await);
    let gitlab = id(&app.post(&format!("/api/companies/{acme}/connections"), json!({ "kind": "GITLAB", "title": "GitLab", "baseUrl": "https://gitlab.example.com" })).await);
    let staging = id(&app.post(&format!("/api/companies/{acme}/connections"), json!({ "kind": "KUBERNETES", "title": "Staging" })).await);
    app.put(&format!("/api/connections/{cluster}/credential"), json!({ "kubeconfig": KUBECONFIG, "context": "prod" })).await;
    app.put(&format!("/api/connections/{gitlab}/credential"), json!({ "token": "glpat-secret" })).await;
    app.put(&format!("/api/projects/{vega}/connections/{cluster}"), json!({ "target": "apps" })).await;
    app.put(&format!("/api/projects/{vega}/connections/{gitlab}"), json!({ "target": "acme/vega" })).await;
    app.put(&format!("/api/projects/{vega}/connections/{staging}"), json!({})).await;

    let context = app.call_tool("rekall_context", json!({ "anchors": "project:vega task:deploy" })).await;

    assert!(context.contains("<connections count=\"3\">"), "{context}");
    assert!(context.contains("`kubectl --context production`, namespace `apps`"));
    assert!(context.contains("GitLab at https://gitlab.example.com, project `acme/vega`"));
    assert!(context.contains("- `staging` (Staging): no credential is stored"));
    for leaked in ["sa-token-123", "glpat-secret", "k8s.example.com", "Q0E=", &format!("connection-{cluster}")] {
        assert!(!context.contains(leaked), "the context carries {leaked}");
    }
}

/// A Jira site behind a proxy with two open issues, which answers only the credential the test stores.
struct FakeJira;

#[async_trait]
impl JiraIssues for FakeJira {
    async fn search(&self, site: &JiraSite, credential: &JiraCredential, _jql: &str, _limit: u16) -> Result<Vec<JiraIssue>, JiraFailure> {
        assert_eq!((site.url.as_str(), site.proxy_url.as_deref(), credential.email()), ("https://example.atlassian.net", Some("http://proxy.example.com:8080"), "ada@example.com"));
        let issue = |key: &str, summary: &str| JiraIssue { key: key.into(), summary: summary.into(), status: Some("To Do".into()), issue_type: Some("Story".into()), description: Some("Steps.".into()) };
        Ok(vec![issue("ENG-1", "Fix login"), issue("ENG-2", "Add export")])
    }
}

#[tokio::test]
async fn a_jira_connection_imports_the_issues_a_project_picks_as_its_tasks() {
    let secrets = Arc::new(InMemorySecretStore::new());
    let options = StartOptions { no_restart: true, secrets: Some(secrets.clone()), jira_issues: Some(Arc::new(FakeJira)), ..StartOptions::default() };
    let app = app_with(&[], options).await;
    let acme = app.a_company("Acme").await;
    let vega = app.a_project(&acme, "vega", "ACTIVE").await;
    let created = app.post(&format!("/api/companies/{acme}/connections"), json!({ "kind": "JIRA", "title": "Jira", "baseUrl": "https://example.atlassian.net", "proxyUrl": " http://proxy.example.com:8080 " })).await;
    assert_eq!(created.str("proxyUrl"), "http://proxy.example.com:8080");
    let jira = id(&created);
    let set = app.put(&format!("/api/connections/{jira}/credential"), json!({ "email": "ada@example.com", "token": "atl-secret" })).await;
    assert_eq!(set.status, 200, "{}", set.text);
    assert!(!set.text.contains("atl-secret"));
    app.put(&format!("/api/projects/{vega}/connections/{jira}"), json!({ "target": "eng" })).await;
    let issues = format!("/api/projects/{vega}/connections/{jira}/jira-issues");

    let listed = app.get(&issues).await;
    assert_eq!(listed.status, 200, "{}", listed.text);
    assert_eq!((listed.list()[0]["url"].as_str(), listed.list()[0]["taskId"].is_null()), (Some("https://example.atlassian.net/browse/ENG-1"), true));

    let imported = app.post(&format!("{issues}/import"), json!({ "keys": ["eng-1"] })).await;
    assert_eq!(imported.status, 200, "{}", imported.text);
    assert_eq!((imported.list()[0]["label"].as_str(), imported.list()[0]["title"].as_str()), (Some("eng-1"), Some("Fix login")));
    assert!(imported.list()[0]["description"].as_str().unwrap().contains("[ENG-1](https://example.atlassian.net/browse/ENG-1)"));
    assert!(!app.get(&issues).await.list()[0]["taskId"].is_null(), "an imported issue names its task");

    assert_eq!(app.post(&format!("{issues}/import"), json!({ "keys": ["OPS-1"] })).await.status, 400);
}

#[tokio::test]
async fn only_a_jira_connection_takes_a_proxy_and_a_blank_one_clears_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let url = format!("/api/companies/{acme}/connections");
    let jira = id(&app.post(&url, json!({ "kind": "JIRA", "title": "Jira", "baseUrl": "https://example.atlassian.net", "proxyUrl": "http://proxy.example.com:8080" })).await);

    assert_eq!(app.post(&url, json!({ "kind": "GITLAB", "title": "GitLab", "baseUrl": "https://gitlab.example.com", "proxyUrl": "http://proxy.example.com:8080" })).await.status, 400);
    let with_password = app.put(&format!("/api/connections/{jira}"), json!({ "proxyUrl": "http://ada:s3cret@proxy.example.com:8080" })).await;
    assert_eq!(with_password.status, 400);
    assert!(!with_password.text.contains("s3cret"));
    assert!(app.put(&format!("/api/connections/{jira}"), json!({ "proxyUrl": "" })).await.get("proxyUrl").is_null());
}
