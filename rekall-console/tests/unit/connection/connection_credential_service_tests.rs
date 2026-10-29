use rekall_common::Instant;
use rekall_store::company;
use rekall_service::connection::InMemorySecretStore;
use rekall_service::EventBus;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

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

struct World {
    credentials: ConnectionCredentialService,
    secrets: Arc<InMemorySecretStore>,
    ctx: Ctx,
    company_id: Id,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());
    World { credentials: ConnectionCredentialService::new(ctx.clone(), secrets.clone()), secrets, ctx, company_id: company.id }
}

impl World {
    async fn connection(&self, kind: ConnectionKind) -> external_connection::Model {
        let base_url = kind.reached_at_typed_url().then(|| "https://example.com".to_string());
        let connection = external_connection::Model::new(self.company_id, format!("{kind}").to_lowercase(), "Conn".into(), kind, base_url, Instant::now());
        connection.clone().into_active_model().insert(&self.ctx.db).await.unwrap();
        connection
    }
}

fn token(value: &str) -> ConnectionCredentialCommand {
    ConnectionCredentialCommand { token: Some(Secret::new(value)), ..ConnectionCredentialCommand::default() }
}

fn kubeconfig(context: &str) -> ConnectionCredentialCommand {
    ConnectionCredentialCommand { kubeconfig: Some(Secret::new(KUBECONFIG)), context: Some(context.into()), ..ConnectionCredentialCommand::default() }
}

#[tokio::test]
async fn a_gitlab_token_goes_to_the_store_and_the_answer_only_says_when() {
    let world = world().await;
    let gitlab = world.connection(ConnectionKind::Gitlab).await;

    let answer = world.credentials.set(gitlab.id, token("glpat-secret")).await.unwrap();

    assert_eq!(world.secrets.read(&gitlab.secret_ref).unwrap().expose(), "glpat-secret");
    assert!(answer.credential_set_at.is_some());
    assert!(!format!("{answer:?}").contains("glpat-secret"));
}

#[tokio::test]
async fn a_kubeconfig_is_stored_as_its_chosen_context_and_the_row_learns_the_server() {
    let world = world().await;
    let cluster = world.connection(ConnectionKind::Kubernetes).await;

    let answer = world.credentials.set(cluster.id, kubeconfig("prod")).await.unwrap();

    let stored = world.secrets.read(&cluster.secret_ref).unwrap();
    assert!(stored.expose().contains("sa-token-123"));
    assert!(!stored.expose().contains("cloud"));
    assert_eq!(answer.base_url.as_deref(), Some("https://k8s.example.com:6443"));
    assert_eq!(answer.kube_context.as_deref(), Some("prod"));
    assert!(!format!("{answer:?}").contains("sa-token-123"));
}

#[tokio::test]
async fn a_context_that_cannot_be_imported_is_refused_and_nothing_is_stored() {
    let world = world().await;
    let cluster = world.connection(ConnectionKind::Kubernetes).await;

    let refused = world.credentials.set(cluster.id, kubeconfig("cloud")).await.unwrap_err();

    assert!(refused.is_illegal_argument());
    assert!(!world.secrets.contains(&cluster.secret_ref));
}

#[tokio::test]
async fn each_kind_takes_only_its_own_credential() {
    let world = world().await;
    let gitlab = world.connection(ConnectionKind::Gitlab).await;
    let cluster = world.connection(ConnectionKind::Kubernetes).await;

    assert!(world.credentials.set(gitlab.id, kubeconfig("prod")).await.unwrap_err().is_illegal_argument());
    assert!(world.credentials.set(cluster.id, token("t")).await.unwrap_err().is_illegal_argument());
    assert!(world.credentials.set(gitlab.id, token("  ")).await.unwrap_err().is_illegal_argument());
}

#[tokio::test]
async fn clearing_empties_the_store_and_the_row_says_no_credential() {
    let world = world().await;
    let gitlab = world.connection(ConnectionKind::Gitlab).await;
    world.credentials.set(gitlab.id, token("glpat-secret")).await.unwrap();

    let answer = world.credentials.clear(gitlab.id).await.unwrap();

    assert!(answer.credential_set_at.is_none());
    assert!(!world.secrets.contains(&gitlab.secret_ref));
}

fn jira_pair(email: &str, value: &str) -> ConnectionCredentialCommand {
    ConnectionCredentialCommand { email: Some(email.into()), token: Some(Secret::new(value)), ..ConnectionCredentialCommand::default() }
}

#[tokio::test]
async fn a_jira_email_and_token_are_stored_as_one_pair() {
    let world = world().await;
    let jira = world.connection(ConnectionKind::Jira).await;

    let answer = world.credentials.set(jira.id, jira_pair("ada@example.com", "atl-secret")).await.unwrap();

    assert!(answer.credential_set_at.is_some());
    assert_eq!(world.secrets.read(&jira.secret_ref).unwrap().expose(), "ada@example.com:atl-secret");
}

#[tokio::test]
async fn a_jira_token_without_an_email_is_stored_as_a_personal_access_token() {
    let world = world().await;
    let jira = world.connection(ConnectionKind::Jira).await;

    world.credentials.set(jira.id, jira_pair("  ", "NjQ0Mj")).await.unwrap();

    assert_eq!(world.secrets.read(&jira.secret_ref).unwrap().expose(), "pat:NjQ0Mj");
}

#[tokio::test]
async fn a_jira_connection_refuses_a_username_in_place_of_an_email_or_a_kubeconfig() {
    let world = world().await;
    let jira = world.connection(ConnectionKind::Jira).await;

    assert!(world.credentials.set(jira.id, jira_pair("vcasale", "atl-secret")).await.unwrap_err().is_illegal_argument());
    assert!(world.credentials.set(jira.id, kubeconfig("prod")).await.unwrap_err().is_illegal_argument());
    assert!(!world.secrets.contains(&jira.secret_ref));
}
