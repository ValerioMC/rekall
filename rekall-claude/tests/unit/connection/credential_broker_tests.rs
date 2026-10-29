use std::os::unix::fs::PermissionsExt;

use rekall_kernel::Instant;
use rekall_store::{company, connection_access, external_connection};
use rekall_core::connection::InMemorySecretStore;
use rekall_core::{Ctx, EventBus};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use super::*;

const KUBECONFIG: &str = "
current-context: prod
clusters:
- name: prod
  cluster: {server: 'https://k8s.example.com:6443', certificate-authority-data: Q0E=}
contexts:
- name: prod
  context: {cluster: prod, user: prod}
users:
- name: prod
  user: {token: sa-token-123}
";

struct World {
    broker: CredentialBroker,
    ctx: Ctx,
    root: tempfile::TempDir,
    launch: TerminalLaunch,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());
    let gitlab = stored(&ctx, &secrets, &company, ConnectionKind::Gitlab, "gitlab", Some("glpat-secret")).await;
    let cluster = stored(&ctx, &secrets, &company, ConnectionKind::Kubernetes, "prod", Some(KUBECONFIG)).await;
    let empty = stored(&ctx, &secrets, &company, ConnectionKind::Kubernetes, "staging", None).await;
    let jira = stored(&ctx, &secrets, &company, ConnectionKind::Jira, "jira", Some("ada@example.com:atl-secret")).await;
    let root = tempfile::tempdir().unwrap();
    let broker = CredentialBroker::new(secrets, ConnectionAccessService::new(ctx.clone()), "http://127.0.0.1:47355".into(), root.path().join("sessions"));
    let launch = launch_with(vec![terminal(&gitlab, Some("acme/vega")), terminal(&cluster, Some("apps")), terminal(&empty, None), terminal(&jira, Some("ENG"))]);
    World { broker, ctx, root, launch }
}

async fn stored(ctx: &Ctx, secrets: &InMemorySecretStore, company: &company::Model, kind: ConnectionKind, label: &str, credential: Option<&str>) -> external_connection::Model {
    let base_url = kind.reached_at_typed_url().then(|| "https://gitlab.example.com".to_string());
    let mut connection = external_connection::Model::new(company.id, label.into(), label.into(), kind, base_url, Instant::now());
    if let Some(credential) = credential {
        secrets.put(&connection.secret_ref, &Secret::new(credential)).unwrap();
        connection.secret_set_at = Some(Instant::now());
    }
    connection.clone().into_active_model().insert(&ctx.db).await.unwrap();
    connection
}

fn terminal(connection: &external_connection::Model, target: Option<&str>) -> TerminalConnection {
    TerminalConnection {
        connection_id: connection.id,
        kind: connection.kind,
        label: connection.label.clone(),
        target: target.map(str::to_string),
        instance_url: connection.base_url.clone().filter(|_| connection.kind.reached_at_typed_url()),
        proxy_url: connection.proxy_url.clone(),
        secret_ref: connection.secret_ref.clone(),
        credential_set: connection.has_credential(),
    }
}

fn launch_with(connections: Vec<TerminalConnection>) -> TerminalLaunch {
    TerminalLaunch {
        project_id: Id::random(),
        task_id: None,
        anchors: "project:vega".into(),
        working_dir: "/work/vega".into(),
        project_label: "vega".into(),
        task_label: None,
        task_title: None,
        connections,
    }
}

fn every_file_in(folder: &Path) -> Vec<(PathBuf, String)> {
    std::fs::read_dir(folder).unwrap().map(|entry| entry.unwrap().path()).map(|path| { let text = std::fs::read_to_string(&path).unwrap(); (path, text) }).collect()
}

async fn log(world: &World) -> Vec<connection_access::Model> {
    connection_access::Entity::find().all(&world.ctx.db).await.unwrap()
}

#[tokio::test]
async fn a_project_with_no_stored_credential_gets_no_session() {
    let world = world().await;
    let terminal_id = Id::random();
    let only_empty = launch_with(world.launch.connections.iter().filter(|c| !c.credential_set).cloned().collect());

    assert_eq!(world.broker.open(terminal_id, &only_empty).await.unwrap(), None);
    assert!(!world.broker.has_session(terminal_id));
}

#[tokio::test]
async fn a_session_writes_private_helpers_that_hold_no_credential() {
    let world = world().await;
    let terminal_id = Id::random();

    let environment = world.broker.open(terminal_id, &world.launch).await.unwrap().unwrap();

    let folder = environment.path_prefix.clone().unwrap();
    assert_eq!(std::fs::metadata(&folder).unwrap().permissions().mode() & 0o777, 0o700);
    let files = every_file_in(&folder);
    assert_eq!(files.len(), 6, "kubeconfig, the exec plugin, the git helper, the GitLab and Jira wrappers and the Claude Code settings");
    for (path, text) in &files {
        assert!(!text.contains("glpat-secret") && !text.contains("sa-token-123") && !text.contains("atl-secret"), "{} holds a credential", path.display());
    }
    assert_eq!(environment.kubeconfig, Some(folder.join("kubeconfig")));
    assert_eq!(environment.claude_settings, Some(folder.join("claude-settings.json")));
    assert!(environment.git_config.iter().any(|(key, value)| key == "credential.https://gitlab.example.com.helper" && value.ends_with(" gitlab")));
    assert!(environment.token.as_deref().is_some_and(|token| token.len() == 64));
}

#[tokio::test]
async fn two_bindings_of_one_gitlab_reach_its_token_by_either_label_through_one_git_helper() {
    let world = world().await;
    let gitlab = world.launch.connections.iter().find(|c| c.kind == ConnectionKind::Gitlab).cloned().unwrap();
    let billing = TerminalConnection { label: "billing".into(), target: Some("acme/billing".into()), ..gitlab.clone() };
    let orders = TerminalConnection { label: "orders".into(), target: Some("acme/orders".into()), ..gitlab };
    let terminal_id = Id::random();

    let environment = world.broker.open(terminal_id, &launch_with(vec![billing, orders])).await.unwrap().unwrap();
    let token = environment.token.clone().unwrap();

    let helpers: Vec<&(String, String)> = environment.git_config.iter().filter(|(key, _)| key == "credential.https://gitlab.example.com.helper").collect();
    assert_eq!(helpers.len(), 2, "one reset and one helper for the host: {helpers:?}");
    for label in ["billing", "orders"] {
        let api = world.broker.hand_out(terminal_id, Some(&token), label, CredentialPurpose::Gitlab).await;
        assert!(matches!(api, HandOut::Granted { ref body, .. } if body.expose() == "PRIVATE-TOKEN: glpat-secret\n"), "{label}");
    }
}

#[tokio::test]
async fn the_terminals_token_gets_the_credential_in_the_shape_each_helper_reads_and_each_is_logged() {
    let world = world().await;
    let terminal_id = Id::random();
    let token = world.broker.open(terminal_id, &world.launch).await.unwrap().unwrap().token.unwrap();

    let git = world.broker.hand_out(terminal_id, Some(&token), "gitlab", CredentialPurpose::Git).await;
    let api = world.broker.hand_out(terminal_id, Some(&token), "gitlab", CredentialPurpose::Gitlab).await;
    let kubectl = world.broker.hand_out(terminal_id, Some(&token), "prod", CredentialPurpose::Kubectl).await;
    let jira = world.broker.hand_out(terminal_id, Some(&token), "jira", CredentialPurpose::Jira).await;

    assert!(matches!(git, HandOut::Granted { ref body, .. } if body.expose() == "username=oauth2\npassword=glpat-secret\n"));
    assert!(matches!(api, HandOut::Granted { ref body, .. } if body.expose() == "PRIVATE-TOKEN: glpat-secret\n"));
    assert!(matches!(kubectl, HandOut::Granted { ref body, content_type: "application/json" } if body.expose().contains("\"token\":\"sa-token-123\"")));
    assert!(matches!(jira, HandOut::Granted { ref body, .. } if body.expose() == "Authorization: Basic YWRhQGV4YW1wbGUuY29tOmF0bC1zZWNyZXQ=\n"));
    assert_eq!(log(&world).await.iter().filter(|access| access.granted).count(), 4);
}

#[tokio::test]
async fn a_wrong_token_or_a_wrong_shape_is_refused_and_logged() {
    let world = world().await;
    let terminal_id = Id::random();
    let token = world.broker.open(terminal_id, &world.launch).await.unwrap().unwrap().token.unwrap();

    let guessed = world.broker.hand_out(terminal_id, Some("0123"), "gitlab", CredentialPurpose::Git).await;
    let missing = world.broker.hand_out(terminal_id, None, "gitlab", CredentialPurpose::Git).await;
    let shape = world.broker.hand_out(terminal_id, Some(&token), "gitlab", CredentialPurpose::Kubectl).await;
    let unset = world.broker.hand_out(terminal_id, Some(&token), "staging", CredentialPurpose::Kubectl).await;

    assert!(matches!(guessed, HandOut::Refused { refusal: Refusal::Forbidden, .. }));
    assert!(matches!(missing, HandOut::Refused { refusal: Refusal::Forbidden, .. }));
    assert!(matches!(shape, HandOut::Refused { refusal: Refusal::WrongPurpose, .. }));
    assert!(matches!(unset, HandOut::Refused { refusal: Refusal::Missing, .. }), "a connection without a credential has no session line");
    let refused = log(&world).await;
    assert_eq!(refused.len(), 3);
    assert!(refused.iter().all(|access| !access.granted && access.reason.is_some()));
}

#[tokio::test]
async fn a_closed_session_answers_nothing_and_leaves_no_folder() {
    let world = world().await;
    let terminal_id = Id::random();
    let environment = world.broker.open(terminal_id, &world.launch).await.unwrap().unwrap();
    let token = environment.token.clone().unwrap();

    world.broker.close(terminal_id);

    assert!(!environment.path_prefix.unwrap().exists());
    let after = world.broker.hand_out(terminal_id, Some(&token), "gitlab", CredentialPurpose::Git).await;
    assert!(matches!(after, HandOut::Refused { refusal: Refusal::Missing, .. }));
    assert!(log(&world).await.is_empty(), "a request from an ended terminal names no connection to log against");
}

#[tokio::test]
async fn a_label_the_project_does_not_bind_is_missing() {
    let world = world().await;
    let terminal_id = Id::random();
    let token = world.broker.open(terminal_id, &world.launch).await.unwrap().unwrap().token.unwrap();
    let answer = world.broker.hand_out(terminal_id, Some(&token), "other", CredentialPurpose::Git).await;
    assert!(matches!(answer, HandOut::Refused { refusal: Refusal::Missing, .. }));
}

#[tokio::test]
async fn folders_left_by_an_earlier_run_are_removed_at_start() {
    let world = world().await;
    let leftover = world.root.path().join("sessions/old-terminal");
    std::fs::create_dir_all(&leftover).unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());

    let _broker = CredentialBroker::new(secrets, ConnectionAccessService::new(world.ctx.clone()), "http://127.0.0.1:1".into(), world.root.path().join("sessions"));

    assert!(!leftover.exists());
}

#[tokio::test]
async fn every_logged_request_is_announced_without_its_credential() {
    let world = world().await;
    let mut events = world.ctx.events.subscribe();
    let terminal_id = Id::random();
    let token = world.broker.open(terminal_id, &world.launch).await.unwrap().unwrap().token.unwrap();

    world.broker.hand_out(terminal_id, Some(&token), "gitlab", CredentialPurpose::Git).await;
    world.broker.hand_out(terminal_id, Some("guess"), "gitlab", CredentialPurpose::Git).await;

    let mut announced = Vec::new();
    while let Ok(event) = events.try_recv() {
        if event.name() == "credential-access" {
            announced.push(event.payload());
        }
    }
    assert_eq!(announced.len(), 2);
    assert_eq!(announced[0]["terminalId"], terminal_id.to_string());
    assert_eq!((announced[0]["granted"].as_bool(), announced[1]["granted"].as_bool()), (Some(true), Some(false)));
    assert!(announced.iter().all(|payload| !payload.to_string().contains("glpat-secret")));
}
