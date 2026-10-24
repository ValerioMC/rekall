//! A terminal's credential helpers run for real against Rekall's endpoint on a live listener: git
//! gets the GitLab token through `git credential fill`, kubectl's plugin prints an `ExecCredential`,
//! and `rekall-gitlab` calls a GitLab API with the token's header. No helper file or environment
//! variable ever holds the credential.

mod support;

use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::Arc;
use std::time::Duration;

use axum::http::HeaderMap;
use axum::routing::get;
use axum::{Json, Router};
use rekall_claude::connection::{CredentialBroker, SessionEnvironment};
use rekall_claude::pty::PtyTerminalManager;
use rekall_claude::terminal::TerminalMode;
use rekall_claude::queue::{RunQueueRunner, RunQueueService};
use rekall_claude::resources::{ClaudeResourceService, SkillGroupService};
use rekall_claude::usage::{ClaudeUsageView, UsageReader};
use rekall_claude::ClaudeState;
use rekall_common::{Id, Instant};
use rekall_model::{connection_access, external_connection, project_connection, ConnectionKind};
use rekall_service::claude::{TerminalLaunch, TerminalTarget};
use rekall_service::connection::{Secret, SecretStore};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use serde_json::{json, Value};
use support::{world, World};

const KUBECONFIG: &str = "
current-context: prod
clusters:
- name: prod
  cluster: {server: 'https://k8s.example.com:6443'}
contexts:
- name: prod
  context: {cluster: prod, user: prod}
users:
- name: prod
  user: {token: sa-token-123}
";

struct NoUsage;

#[async_trait::async_trait]
impl UsageReader for NoUsage {
    async fn current(&self) -> ClaudeUsageView {
        ClaudeUsageView::unauthenticated()
    }

    async fn refresh(&self) -> ClaudeUsageView {
        ClaudeUsageView::unauthenticated()
    }
}

/// A GitLab that answers `/api/v4/projects/{id}` only to the right token, echoing the id it was asked for.
async fn fake_gitlab() -> String {
    let router = Router::new().route(
        "/api/v4/projects/{id}",
        get(|headers: HeaderMap, axum::extract::Path(id): axum::extract::Path<String>| async move {
            match headers.get("PRIVATE-TOKEN").and_then(|value| value.to_str().ok()) {
                Some("glpat-secret") => Ok(Json(json!({ "path_with_namespace": id }))),
                _ => Err(axum::http::StatusCode::UNAUTHORIZED),
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

/// Rekall's terminal routes on a live listener; the broker's helpers ask that listener.
async fn serve(world: &World) -> Arc<CredentialBroker> {
    serve_terminals(world).await.0
}

async fn serve_terminals(world: &World) -> (Arc<CredentialBroker>, Arc<PtyTerminalManager>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let broker = world.broker(&format!("http://{address}"));
    let terminals = world.terminals_with(8, broker.clone());
    let usage: Arc<dyn UsageReader> = Arc::new(NoUsage);
    let queue = RunQueueService::new(world.services.ctx.clone());
    let runner = RunQueueRunner::start(
        queue.clone(),
        terminals.clone(),
        usage.clone(),
        world.services.task_work.clone(),
        world.services.steps.clone(),
        &world.events,
        rekall_service::system_clock(),
        Duration::from_secs(60),
        Duration::from_secs(1),
    );
    let resources = ClaudeResourceService::new(world.folder.path().join(".claude"));
    let skill_groups = SkillGroupService::new(world.services.ctx.clone());
    let state = ClaudeState { terminals: terminals.clone(), usage, queue, runner, resources, skill_groups, broker: broker.clone() };
    let app = rekall_claude::router(state).into_make_service_with_connect_info::<SocketAddr>();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (broker, terminals)
}

/// A project binding a GitLab instance at `gitlab_url` and a cluster, both with a stored credential.
async fn launch(world: &World, gitlab_url: &str) -> TerminalLaunch {
    launch_with_cluster(world, gitlab_url, KUBECONFIG).await
}

async fn launch_with_cluster(world: &World, gitlab_url: &str, kubeconfig: &'static str) -> TerminalLaunch {
    let project = world.project("vega", Some(world.folder.path())).await;
    for (kind, label, base_url, credential, target) in [
        (ConnectionKind::Gitlab, "gitlab", Some(gitlab_url.to_string()), "glpat-secret", Some("acme/vega")),
        (ConnectionKind::Kubernetes, "prod", None, kubeconfig, Some("apps")),
    ] {
        let mut connection = external_connection::Model::new(project.company_id, label.into(), label.into(), kind, base_url, Instant::now());
        world.secrets.put(&connection.secret_ref, &Secret::new(credential)).unwrap();
        connection.secret_set_at = Some(Instant::now());
        connection.clone().into_active_model().insert(world.db()).await.unwrap();
        project_connection::Model::new(project.id, connection.id, label.into(), target.map(str::to_string), Instant::now()).into_active_model().insert(world.db()).await.unwrap();
    }
    world.services.terminal_launch.resolve(TerminalTarget::Project(project.id)).await.unwrap()
}

fn environment_of(session: &SessionEnvironment) -> HashMap<String, String> {
    let mut environment = HashMap::from([("PATH".to_string(), "/usr/bin:/bin".to_string()), ("HOME".to_string(), "/nonexistent".to_string())]);
    session.apply(&mut environment);
    environment
}

/// Runs `program` with exactly `environment`, feeding `input` on stdin, off the async runtime.
async fn run(program: String, args: Vec<String>, environment: HashMap<String, String>, input: &'static str, folder: &Path) -> Output {
    let folder = folder.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut child = Command::new(program)
            .args(args)
            .env_clear()
            .envs(environment)
            .current_dir(folder)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        child.wait_with_output().unwrap()
    })
    .await
    .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[tokio::test]
async fn git_fills_the_gitlab_credential_through_the_helper_and_nothing_on_disk_holds_it() {
    let world = world().await;
    let broker = serve(&world).await;
    let launch = launch(&world, "https://gitlab.example.com").await;
    let session = broker.open(Id::random(), &launch).await.unwrap().unwrap();
    let environment = environment_of(&session);
    assert!(environment.values().all(|value| !value.contains("glpat-secret")), "the environment holds the token");

    let filled = run("git".into(), vec!["credential".into(), "fill".into()], environment, "protocol=https\nhost=gitlab.example.com\n\n", world.folder.path()).await;

    assert!(filled.status.success(), "{}", text(&filled.stderr));
    let answer = text(&filled.stdout);
    assert!(answer.contains("username=oauth2"), "{answer}");
    assert!(answer.contains("password=glpat-secret"), "{answer}");
    for entry in std::fs::read_dir(session.path_prefix.unwrap()).unwrap() {
        let file = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        assert!(!file.contains("glpat-secret") && !file.contains("sa-token-123"));
    }
}

#[tokio::test]
async fn kubectls_plugin_prints_an_exec_credential_for_the_terminal_and_nothing_with_a_wrong_token() {
    let world = world().await;
    let broker = serve(&world).await;
    let launch = launch(&world, "https://gitlab.example.com").await;
    let session = broker.open(Id::random(), &launch).await.unwrap().unwrap();
    let plugin = session.path_prefix.clone().unwrap().join("rekall-kube-credential").display().to_string();

    let granted = run(plugin.clone(), vec!["prod".into()], environment_of(&session), "", world.folder.path()).await;
    let mut forged = environment_of(&session);
    forged.insert("REKALL_CREDENTIAL_TOKEN".into(), "guess".into());
    let refused = run(plugin, vec!["prod".into()], forged, "", world.folder.path()).await;

    assert!(granted.status.success(), "{}", text(&granted.stderr));
    let credential: Value = serde_json::from_slice(&granted.stdout).unwrap();
    assert_eq!((credential["kind"].as_str(), credential["status"]["token"].as_str()), (Some("ExecCredential"), Some("sa-token-123")));
    assert!(!refused.status.success());
    assert!(text(&refused.stderr).contains("not this terminal's"), "{}", text(&refused.stderr));
    assert!(refused.stdout.is_empty());
}

#[tokio::test]
async fn rekall_gitlab_calls_the_api_with_the_token_and_the_project_path_in_place_of_project() {
    let world = world().await;
    let broker = serve(&world).await;
    let gitlab = fake_gitlab().await;
    let launch = launch(&world, &gitlab).await;
    let session = broker.open(Id::random(), &launch).await.unwrap().unwrap();
    let wrapper = session.path_prefix.clone().unwrap().join("rekall-gitlab").display().to_string();

    let answer = run(wrapper, ["--connection", "gitlab", "GET", "/projects/:project"].map(String::from).to_vec(), environment_of(&session), "", world.folder.path()).await;

    assert!(answer.status.success(), "{}", text(&answer.stderr));
    let project: Value = serde_json::from_slice(&answer.stdout).unwrap();
    assert_eq!(project["path_with_namespace"], "acme/vega");
}

#[tokio::test]
async fn after_the_session_closes_the_helpers_get_nothing() {
    let world = world().await;
    let broker = serve(&world).await;
    let launch = launch(&world, "https://gitlab.example.com").await;
    let terminal_id = Id::random();
    let session = broker.open(terminal_id, &launch).await.unwrap().unwrap();
    let environment = environment_of(&session);
    let helper = session.path_prefix.clone().unwrap().join("rekall-git-credential");
    let copy = world.home.path().join("kept-helper");
    std::fs::copy(&helper, &copy).unwrap();

    broker.close(terminal_id);
    let after = run(copy.display().to_string(), vec!["gitlab".into(), "get".into()], environment, "protocol=https\nhost=gitlab.example.com\n\n", world.folder.path()).await;

    assert!(!after.status.success());
    assert!(!text(&after.stdout).contains("glpat-secret"));
}

/// A cluster nothing listens for: kubectl has to fetch the credential before it finds that out.
const UNREACHABLE_CLUSTER: &str = "
current-context: prod
clusters:
- name: prod
  cluster: {server: 'https://127.0.0.1:1'}
contexts:
- name: prod
  context: {cluster: prod, user: prod}
users:
- name: prod
  user: {token: sa-token-123}
";

fn kubectl_installed() -> bool {
    Command::new("kubectl").args(["version", "--client"]).output().is_ok_and(|output| output.status.success())
}

#[tokio::test]
async fn kubectl_reads_the_session_kubeconfig_and_asks_the_plugin_for_the_credential() {
    if !kubectl_installed() {
        return;
    }
    let world = world().await;
    let broker = serve(&world).await;
    let launch = launch_with_cluster(&world, "https://gitlab.example.com", UNREACHABLE_CLUSTER).await;
    let session = broker.open(Id::random(), &launch).await.unwrap().unwrap();
    let mut environment = environment_of(&session);
    environment.insert("PATH".into(), "/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin".into());

    let namespace = run("kubectl".into(), ["config", "view", "--minify", "-o", "jsonpath={.contexts[0].context.namespace}"].map(String::from).to_vec(), environment.clone(), "", world.folder.path()).await;
    let raw = run("kubectl".into(), ["config", "view", "--raw"].map(String::from).to_vec(), environment.clone(), "", world.folder.path()).await;
    run("kubectl".into(), ["--context", "prod", "get", "--raw", "/version", "--request-timeout=3s"].map(String::from).to_vec(), environment, "", world.folder.path()).await;

    assert_eq!(text(&namespace.stdout), "apps");
    assert!(!text(&raw.stdout).contains("sa-token-123"), "the session kubeconfig printed raw holds no credential");
    let log = connection_access::Entity::find().all(world.db()).await.unwrap();
    assert!(log.iter().any(|access| access.granted), "kubectl never asked the plugin");
}

#[tokio::test]
async fn a_terminal_on_a_project_with_connections_has_a_session_exactly_as_long_as_it_lives() {
    let world = world().await;
    let (broker, terminals) = serve_terminals(&world).await;
    let launch = launch(&world, "https://gitlab.example.com").await;

    let view = terminals.open(TerminalTarget::Project(launch.project_id), None, false, None, None, TerminalMode::Explain { request: "how".into() }).await.unwrap();
    assert!(broker.has_session(view.id));
    let named: Vec<(String, bool)> = view.connections.iter().map(|connection| (connection.label.clone(), connection.credential_set)).collect();
    assert_eq!(named, [("gitlab".to_string(), true), ("prod".to_string(), true)]);
    assert!(!serde_json::to_string(&view).unwrap().contains("connection-"), "the view names no keychain account");

    terminals.close(view.id, "test").await;
    assert!(!broker.has_session(view.id));
}
