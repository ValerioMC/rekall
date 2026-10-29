use std::sync::Mutex;

use async_trait::async_trait;
use rekall_kernel::Instant;
use rekall_store::company;
use crate::connection::InMemorySecretStore;
use crate::EventBus;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

const KUBECONFIG: &str = "
current-context: prod
clusters:
- name: prod
  cluster: {server: 'https://k8s.example.com'}
contexts:
- name: prod
  context: {cluster: prod, user: prod}
users:
- name: prod
  user: {token: sa-token-123}
";

/// Answers what it was told to and remembers which server it was sent to.
struct FakeProbe {
    outcome: ProbeOutcome,
    asked: Mutex<Vec<String>>,
}

#[async_trait]
impl ConnectionProbe for FakeProbe {
    async fn gitlab(&self, base_url: &str, _token: &Secret) -> ProbeOutcome {
        self.asked.lock().unwrap().push(base_url.to_string());
        self.outcome.clone()
    }

    async fn kubernetes(&self, access: &KubeAccess) -> ProbeOutcome {
        self.asked.lock().unwrap().push(access.server.clone());
        self.outcome.clone()
    }

    async fn jira(&self, site: &JiraSite, credential: &JiraCredential) -> ProbeOutcome {
        let proxy = site.proxy_url.as_deref().unwrap_or("no proxy");
        self.asked.lock().unwrap().push(format!("{} via {proxy} as {}", site.url, credential.email().unwrap_or("a personal access token")));
        self.outcome.clone()
    }
}

struct World {
    checks: ConnectionCheckService,
    probe: Arc<FakeProbe>,
    secrets: Arc<InMemorySecretStore>,
    ctx: Ctx,
    company_id: Id,
}

async fn world(outcome: ProbeOutcome) -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());
    let probe = Arc::new(FakeProbe { outcome, asked: Mutex::new(Vec::new()) });
    World { checks: ConnectionCheckService::new(ctx.clone(), secrets.clone(), probe.clone()), probe, secrets, ctx, company_id: company.id }
}

impl World {
    async fn connection(&self, kind: ConnectionKind, credential: Option<&str>) -> Id {
        let base_url = match kind {
            ConnectionKind::Gitlab => Some("https://gitlab.example.com".to_string()),
            ConnectionKind::Jira => Some("https://example.atlassian.net".to_string()),
            ConnectionKind::Kubernetes => None,
        };
        let mut connection = external_connection::Model::new(self.company_id, "conn".into(), "Conn".into(), kind, base_url, Instant::now());
        connection.proxy_url = (kind == ConnectionKind::Jira).then(|| "http://proxy.example.com:8080".to_string());
        if let Some(credential) = credential {
            self.secrets.put(&connection.secret_ref, &Secret::new(credential)).unwrap();
            connection.secret_set_at = Some(Instant::now());
        }
        connection.clone().into_active_model().insert(&self.ctx.db).await.unwrap();
        connection.id
    }
}

#[tokio::test]
async fn a_connection_without_a_credential_is_reported_without_asking_the_server() {
    let world = world(ProbeOutcome::Accepted { identity: None, version: None }).await;
    let id = world.connection(ConnectionKind::Gitlab, None).await;

    let check = world.checks.check(id).await.unwrap();

    assert!(!check.ok);
    assert!(check.detail.contains("No credential"));
    assert!(world.probe.asked.lock().unwrap().is_empty());
}

#[tokio::test]
async fn an_accepted_gitlab_token_names_who_it_signs_in_as() {
    let world = world(ProbeOutcome::Accepted { identity: Some("deployer".into()), version: None }).await;
    let id = world.connection(ConnectionKind::Gitlab, Some("glpat-secret")).await;

    let check = world.checks.check(id).await.unwrap();

    assert!(check.ok);
    assert_eq!(check.detail, "Signed in as deployer.");
    assert_eq!(*world.probe.asked.lock().unwrap(), ["https://gitlab.example.com"]);
}

#[tokio::test]
async fn a_kubernetes_check_asks_the_server_of_the_stored_kubeconfig() {
    let world = world(ProbeOutcome::Accepted { identity: Some("system:serviceaccount:apps:deployer".into()), version: Some("v1.30.2".into()) }).await;
    let id = world.connection(ConnectionKind::Kubernetes, Some(KUBECONFIG)).await;

    let check = world.checks.check(id).await.unwrap();

    assert!(check.ok);
    assert_eq!(check.detail, "Signed in as system:serviceaccount:apps:deployer, server v1.30.2.");
    assert_eq!(*world.probe.asked.lock().unwrap(), ["https://k8s.example.com"]);
}

#[tokio::test]
async fn a_rejected_or_unreachable_server_is_said_without_the_credential() {
    for outcome in [ProbeOutcome::Rejected { status: 401 }, ProbeOutcome::Unreachable { reason: "connection refused".into() }] {
        let world = world(outcome).await;
        let id = world.connection(ConnectionKind::Kubernetes, Some(KUBECONFIG)).await;

        let check = world.checks.check(id).await.unwrap();

        assert!(!check.ok);
        assert!(!check.detail.contains("sa-token-123"), "{}", check.detail);
    }
}

#[tokio::test]
async fn a_rejection_says_to_replace_the_credential() {
    let world = world(ProbeOutcome::Rejected { status: 403 }).await;
    let id = world.connection(ConnectionKind::Gitlab, Some("glpat-secret")).await;
    assert!(world.checks.check(id).await.unwrap().detail.contains("HTTP 403"));
}

#[tokio::test]
async fn a_jira_check_asks_the_site_through_its_proxy_with_the_stored_email_and_token() {
    let world = world(ProbeOutcome::Accepted { identity: Some("Ada Lovelace".into()), version: None }).await;
    let id = world.connection(ConnectionKind::Jira, Some("ada@example.com:atl-secret")).await;

    let check = world.checks.check(id).await.unwrap();

    assert!(check.ok);
    assert_eq!(check.detail, "Signed in as Ada Lovelace.");
    assert_eq!(*world.probe.asked.lock().unwrap(), ["https://example.atlassian.net via http://proxy.example.com:8080 as ada@example.com"]);
}
