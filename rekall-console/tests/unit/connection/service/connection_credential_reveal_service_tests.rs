use rekall_kernel::{Instant, RekallError};
use rekall_core::connection::{InMemorySecretStore, Secret};
use rekall_core::EventBus;
use rekall_store::{external_connection, ConnectionKind};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use crate::connection::ConnectionCredentialCommand;
use crate::connection::ConnectionCredentialService;

use super::*;

const KUBECONFIG: &str = "
clusters:
- name: c
  cluster: {server: 'https://k8s.example.com:6443', certificate-authority-data: Q0E=}
contexts:
- name: prod
  context: {cluster: c, user: u}
users:
- name: u
  user: {token: sa-token-123}
";

struct World {
    credentials: ConnectionCredentialService,
    reveals: ConnectionCredentialRevealService,
    ctx: Ctx,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let secrets = Arc::new(InMemorySecretStore::new());
    World {
        credentials: ConnectionCredentialService::new(ctx.clone(), secrets.clone()),
        reveals: ConnectionCredentialRevealService::new(ctx.clone(), secrets),
        ctx,
    }
}

impl World {
    async fn connection(&self, kind: ConnectionKind) -> external_connection::Model {
        let base_url = kind.reached_at_typed_url().then(|| "https://example.com".to_string());
        let connection = external_connection::Model::new(format!("{kind}").to_lowercase(), "Conn".into(), kind, base_url, Instant::now());
        connection.clone().into_active_model().insert(&self.ctx.db).await.unwrap();
        connection
    }
}

fn token(value: &str) -> ConnectionCredentialCommand {
    ConnectionCredentialCommand { token: Some(Secret::new(value)), ..ConnectionCredentialCommand::default() }
}

#[tokio::test]
async fn a_stored_token_is_read_back() {
    let world = world().await;
    let grafana = world.connection(ConnectionKind::Grafana).await;
    world.credentials.set(grafana.id, token("glsa_secret")).await.unwrap();

    let revealed = world.reveals.reveal(grafana.id).await.unwrap();

    assert_eq!(revealed.token.unwrap().expose(), "glsa_secret");
    assert!(revealed.kubeconfig.is_none());
    assert!(revealed.email.is_none());
}

#[tokio::test]
async fn a_jira_cloud_credential_comes_back_as_its_email_and_token() {
    let world = world().await;
    let jira = world.connection(ConnectionKind::Jira).await;
    let command = ConnectionCredentialCommand { email: Some("me@example.com".into()), token: Some(Secret::new("api-token")), ..ConnectionCredentialCommand::default() };
    world.credentials.set(jira.id, command).await.unwrap();

    let revealed = world.reveals.reveal(jira.id).await.unwrap();

    assert_eq!(revealed.token.unwrap().expose(), "api-token");
    assert_eq!(revealed.email.as_deref(), Some("me@example.com"));
}

#[tokio::test]
async fn a_jira_data_center_token_comes_back_without_an_email() {
    let world = world().await;
    let jira = world.connection(ConnectionKind::Jira).await;
    world.credentials.set(jira.id, token("pat-token")).await.unwrap();

    let revealed = world.reveals.reveal(jira.id).await.unwrap();

    assert_eq!(revealed.token.unwrap().expose(), "pat-token");
    assert!(revealed.email.is_none());
}

#[tokio::test]
async fn a_connection_with_no_credential_is_not_found() {
    let world = world().await;
    let gitlab = world.connection(ConnectionKind::Gitlab).await;

    let refused = world.reveals.reveal(gitlab.id).await.unwrap_err();

    assert!(matches!(refused, RekallError::NotFound(_)), "{refused:?}");
}

#[tokio::test]
async fn a_revealed_credential_prints_as_stars() {
    let world = world().await;
    let gitlab = world.connection(ConnectionKind::Gitlab).await;
    world.credentials.set(gitlab.id, token("glpat-secret")).await.unwrap();

    let revealed = world.reveals.reveal(gitlab.id).await.unwrap();

    assert!(!format!("{revealed:?}").contains("glpat-secret"));
}

#[tokio::test]
async fn a_stored_kubeconfig_is_read_back_as_the_chosen_context() {
    let world = world().await;
    let cluster = world.connection(ConnectionKind::Kubernetes).await;
    let command = ConnectionCredentialCommand { kubeconfig: Some(Secret::new(KUBECONFIG)), context: Some("prod".into()), ..ConnectionCredentialCommand::default() };
    world.credentials.set(cluster.id, command).await.unwrap();

    let revealed = world.reveals.reveal(cluster.id).await.unwrap();

    assert!(revealed.kubeconfig.unwrap().expose().contains("sa-token-123"));
    assert!(revealed.token.is_none());
}
