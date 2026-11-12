//! The parts `Instance::build` assembles: each port taken from the start options or built the way
//! production builds it, the terminal and chat sessions, the run queue and the router.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::{middleware, Router};
use rekall_core::check::CheckRunner;
use rekall_core::claude::{TerminalHost, UsageReader};
use rekall_core::connection::{ConnectionProbe, JiraIssues, MigratingSecretStore, RememberingSecretStore, SecretStore, TaskTracker, TaskTrackerService};
use rekall_core::queue::RunQueueService;
use rekall_core::version::ReleaseFeed;
use rekall_core::vulnerability::AdvisoryFeed;
use rekall_core::worktree::{TaskPullRequestService, TaskWorktreeService, WorktreeHost, WorktreeLayout};
use rekall_core::{Clock, Ctx, EventBus, Services};
use rekall_http::shared::error::{framework, render_errors};
use rekall_http::ApiState;
use rekall_infra::chat::ChatSessionManager;
use rekall_infra::check::ShellCheckRunner;
use rekall_infra::cli::ClaudeCli;
use rekall_infra::connection::CredentialBroker;
use rekall_infra::credentials::ClaudeCredentials;
use rekall_infra::jira::HttpJiraIssues;
use rekall_infra::login_shell::LoginShellEnvironment;
use rekall_infra::probes::HttpConnectionProbe;
use rekall_infra::pty::{PtyTerminalManager, PtyTerminals, SessionRelay};
use rekall_infra::queue::RunQueueRunner;
use rekall_infra::secret::{FileSecretStore, KeychainSecretStore, SECRETS_FILE};
use rekall_infra::session::{SessionHosts, WorkMarker};
use rekall_infra::tracker::{ConnectionPullRequestHost, HttpTaskTracker};
use rekall_infra::usage::ClaudeUsageService;
use rekall_infra::version::GithubReleaseFeed;
use rekall_infra::vulnerability::GithubAdvisoryFeed;
use rekall_infra::worktree::{GitWorktreeHost, WorktreeCleaner};
use rekall_infra::ClaudeConfig;
use rekall_core::telemetry::TelemetrySettingsService;
use rekall_core::connection::ConnectionAccessService;

use crate::config::AppConfig;
use crate::security::{self, LocalAccess};
use crate::spa::{self, Assets};
use crate::actuator;

use super::StartOptions;

/// Under `rekall.home`: one folder per project and task for the git worktrees the run queue works in.
const WORKTREES_FOLDER: &str = "worktrees";

/// Under `rekall.home`: one folder of credential helpers per live terminal, emptied at every start.
const CREDENTIAL_SESSIONS_FOLDER: &str = "terminal-credentials";

const MERGE_SESSION_CHECK: Duration = Duration::from_secs(5);

pub(super) fn check_runner_of(options: &StartOptions, login_shell: &Arc<LoginShellEnvironment>) -> Arc<dyn CheckRunner> {
    match &options.check_runner {
        Some(runner) => runner.clone(),
        None => Arc::new(ShellCheckRunner::new(login_shell.clone())),
    }
}

pub(super) fn secrets_of(config: &AppConfig, options: &StartOptions) -> Arc<dyn SecretStore> {
    match &options.secrets {
        Some(secrets) => secrets.clone(),
        None => Arc::new(MigratingSecretStore::new(
            Arc::new(FileSecretStore::new(config.home.join(SECRETS_FILE))),
            Arc::new(RememberingSecretStore::new(Arc::new(KeychainSecretStore::new()))),
        )),
    }
}

pub(super) fn jira_issues_of(options: &StartOptions) -> Arc<dyn JiraIssues> {
    match &options.jira_issues {
        Some(jira) => jira.clone(),
        None => Arc::new(HttpJiraIssues::new()),
    }
}

pub(super) fn task_tracker_of(options: &StartOptions) -> Arc<dyn TaskTracker> {
    match &options.task_tracker {
        Some(tracker) => tracker.clone(),
        None => Arc::new(HttpTaskTracker::new()),
    }
}

pub(super) fn advisory_feed_of(options: &StartOptions, secrets: &Arc<dyn SecretStore>) -> Arc<dyn AdvisoryFeed> {
    match &options.advisory_feed {
        Some(feed) => feed.clone(),
        None => Arc::new(GithubAdvisoryFeed::new(secrets.clone())),
    }
}

pub(super) fn usage_of(options: &StartOptions, claude_config: &ClaudeConfig, clock: &Clock) -> Arc<dyn UsageReader> {
    match &options.usage {
        Some(usage) => usage.clone(),
        None => Arc::new(ClaudeUsageService::new(
            Arc::new(ClaudeCredentials::new(claude_config.home.clone())),
            &claude_config.usage_url,
            clock.clone(),
        )),
    }
}

pub(super) fn release_feed_of(options: &StartOptions, config: &AppConfig) -> Arc<dyn ReleaseFeed> {
    match &options.release_feed {
        Some(feed) => feed.clone(),
        None => Arc::new(GithubReleaseFeed::new(&config.update_check.url)),
    }
}

pub(super) fn connection_probe_of(options: &StartOptions) -> Arc<dyn ConnectionProbe> {
    match &options.connection_probe {
        Some(probe) => probe.clone(),
        None => Arc::new(HttpConnectionProbe::new()),
    }
}

/// The terminal and chat sessions one instance hosts, and the broker that hands each its credentials.
pub(super) struct Sessions {
    pub broker: Arc<CredentialBroker>,
    pub terminals: Arc<PtyTerminalManager>,
    pub chats: Arc<ChatSessionManager>,
    pub pty_terminals: Arc<PtyTerminals>,
    pub hosts: Arc<dyn TerminalHost>,
}

pub(super) fn open_sessions(
    config: &AppConfig,
    ctx: &Ctx,
    services: &Services,
    secrets: &Arc<dyn SecretStore>,
    login_shell: &Arc<LoginShellEnvironment>,
    served_port: u16,
) -> Sessions {
    let claude_config = &config.claude;
    let cli = ClaudeCli::new(claude_config.cli_path.as_deref(), claude_config.home.clone(), Arc::new(login_shell.clone()));
    let broker = Arc::new(CredentialBroker::new(
        secrets.clone(),
        ConnectionAccessService::new(ctx.clone()),
        format!("http://127.0.0.1:{served_port}"),
        config.home.join(CREDENTIAL_SESSIONS_FOLDER),
    ));
    let work_marker = WorkMarker::new(services.steps.clone(), services.review.clone(), services.step_snapshots.clone(), services.feedback.clone());
    let terminals = PtyTerminalManager::new(
        services.terminal_launch.clone(),
        cli.clone(),
        work_marker.clone(),
        broker.clone(),
        TelemetrySettingsService::new(ctx.clone(), secrets.clone()),
        claude_config,
    );
    let chats = ChatSessionManager::new(
        services.terminal_launch.clone(),
        cli,
        work_marker,
        broker.clone(),
        TelemetrySettingsService::new(ctx.clone(), secrets.clone()),
        claude_config,
    );
    let pty_terminals = Arc::new(PtyTerminals::new(terminals.clone()));
    let hosts: Arc<dyn TerminalHost> = Arc::new(SessionHosts::start(pty_terminals.clone(), chats.clone(), claude_config.max_sessions));
    Sessions { broker, terminals, chats, pty_terminals, hosts }
}

/// The run queue's worker, and the worktrees it and the console's branch actions share.
pub(super) struct QueueParts {
    pub runner: RunQueueRunner,
    pub worktrees: TaskWorktreeService,
}

pub(super) fn start_queue(
    config: &AppConfig,
    ctx: &Ctx,
    events: &EventBus,
    services: &Services,
    task_tracker: &TaskTrackerService,
    sessions: &Arc<dyn TerminalHost>,
    usage: &Arc<dyn UsageReader>,
) -> QueueParts {
    let claude_config = &config.claude;
    let queue = RunQueueService::new(ctx.clone());
    let layout = WorktreeLayout::new(config.home.join(WORKTREES_FOLDER));
    let git: Arc<dyn WorktreeHost> = Arc::new(GitWorktreeHost::new(layout.clone()));
    let worktrees = TaskWorktreeService::new(ctx.clone(), git.clone(), layout.clone());
    let pull_requests = TaskPullRequestService::new(ctx.clone(), task_tracker.clone(), git, Arc::new(ConnectionPullRequestHost::new()), layout);
    WorktreeCleaner::start(events, services.task_work.clone(), worktrees.clone(), pull_requests, sessions.clone(), MERGE_SESSION_CHECK);
    let runner = RunQueueRunner::start(
        queue.clone(),
        sessions.clone(),
        usage.clone(),
        services.task_work.clone(),
        services.steps.clone(),
        worktrees.clone(),
        events,
        ctx.clock.clone(),
        claude_config.run_queue_tick,
        claude_config.run_queue_settle_grace,
    );
    SessionRelay::start(sessions.clone(), services.task_work.clone(), queue, events, claude_config.run_queue_settle_grace);
    QueueParts { runner, worktrees }
}

/// Every module's routes behind the local-access guard and the error renderer. The routes sit
/// behind a fallback so the layers see each response whole.
pub(super) fn assemble_router(config: &AppConfig, api: ApiState, mcp: rekall_mcp::McpController, database: sea_orm::DatabaseConnection) -> Router {
    let routes = Router::new()
        .merge(rekall_http::router(api))
        .merge(rekall_mcp::router(mcp))
        .merge(actuator::routes(database))
        .merge(spa::routes(Assets::new(config.ui_dist.clone())))
        .method_not_allowed_fallback(|| async { framework(StatusCode::METHOD_NOT_ALLOWED) });
    Router::new()
        .fallback_service(routes)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(middleware::from_fn_with_state(LocalAccess::new(config.remote_access), security::guard))
        .layer(middleware::from_fn(render_errors))
}
