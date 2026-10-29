use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::{middleware, Router};
use rekall_web::error::{framework, render_errors};
use rekall_api::{ApiState, StepEventStream};
use rekall_infra::cli::ClaudeCli;
use rekall_infra::connection::CredentialBroker;
use rekall_infra::credentials::ClaudeCredentials;
use rekall_infra::login_shell::LoginShellEnvironment;
use rekall_infra::pty::PtyTerminalManager;
use rekall_core::queue::RunQueueService;
use rekall_infra::queue::RunQueueRunner;
use rekall_core::skill::SkillGroupService;
use rekall_infra::resources::ClaudeResourceService;
use rekall_infra::transcript::{TranscriptFolder, TranscriptScanner};
use rekall_infra::usage::{ClaudeUsageService, UsageReader};
use rekall_infra::ClaudeState;
use rekall_store::Database;
use rekall_core::connection::{ConnectionAccessService, JiraIssues, RememberingSecretStore, SecretStore};
use rekall_core::timeentry::IDLE_AFTER;
use rekall_core::vulnerability::{AdvisoryFeed, ScanSettingsService, VulnerabilityScanService};
use rekall_core::{Ctx, EventBus, Services};
use tracing::warn;

use rekall_infra::backup::{self, DatabaseBackupService, DatabaseLocation, DatabaseRestoreService};
use rekall_infra::installer::{self, ClaudeCodeInstaller};
use rekall_infra::bootstrap::location::SetupStatus;
use rekall_infra::bootstrap::registry::DatabaseRegistryStore;
use rekall_infra::bootstrap::settings::{self, SettingsState};
use crate::config::AppConfig;
use rekall_infra::restart::Restarter;
use rekall_core::connection::{ConnectionCheckService, ConnectionProbe};
use rekall_infra::jira::HttpJiraIssues;
use rekall_infra::probes::{self, HttpConnectionProbe};
use rekall_infra::secret::KeychainSecretStore;
use crate::security::{self, LocalAccess};
use crate::spa::{self, Assets};
use crate::actuator;
use rekall_infra::version::{self, GithubReleaseFeed, ReleaseFeed, UpdateChecker, RUNNING_VERSION};
use rekall_infra::vulnerability::GithubAdvisoryFeed;

use super::StartOptions;

/// Under `rekall.home`: one folder of credential helpers per live terminal, emptied at every start.
const CREDENTIAL_SESSIONS_FOLDER: &str = "terminal-credentials";

/// How often open timers are checked against [`IDLE_AFTER`].
const IDLE_SWEEP_EVERY: Duration = Duration::from_secs(60);

/// When the first scheduled vulnerability scan runs after a start, and how often after that: soon
/// enough that a critical advisory published overnight is a warning by the morning, rarely enough to
/// stay inside GitHub's hourly allowance for requests without a token.
const VULNERABILITY_SCAN_AFTER: Duration = Duration::from_secs(5 * 60);
const VULNERABILITY_SCAN_EVERY: Duration = Duration::from_secs(6 * 60 * 60);

/// One start of the application on one database.
pub struct Instance {
    pub database: Database,
    pub services: Services,
    pub status: SetupStatus,
    pub(super) router: Router,
    stream: Arc<StepEventStream>,
    terminals: Arc<PtyTerminalManager>,
    usage: Arc<dyn UsageReader>,
    runner: RunQueueRunner,
    backups: DatabaseBackupService,
    login_shell: Arc<LoginShellEnvironment>,
    sweep_minutes: u64,
    idle_sweep: Mutex<Option<tokio::task::JoinHandle<()>>>,
    transcripts: Arc<TranscriptScanner>,
    transcript_scan_every: Duration,
    transcript_scan: Mutex<Option<tokio::task::JoinHandle<()>>>,
    vulnerability_scans: VulnerabilityScanService,
    vulnerability_scan: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Instance {
    pub fn build(
        config: &AppConfig,
        database: Database,
        status: SetupStatus,
        restarter: Restarter,
        served_port: u16,
        options: &StartOptions,
    ) -> Self {
        let events = EventBus::new();
        let mut ctx = Ctx::new(database.conn.clone(), events.clone());
        if let Some(clock) = &options.clock {
            ctx.clock = clock.clone();
        }
        let clock = ctx.clock.clone();
        let services = Services::new(ctx.clone());

        let secrets: Arc<dyn SecretStore> = match &options.secrets {
            Some(secrets) => secrets.clone(),
            None => Arc::new(RememberingSecretStore::new(Arc::new(KeychainSecretStore::new()))),
        };
        let stream = Arc::new(StepEventStream::new(events.clone()));
        let jira_issues: Arc<dyn JiraIssues> = match &options.jira_issues {
            Some(jira) => jira.clone(),
            None => Arc::new(HttpJiraIssues::new()),
        };
        let advisory_feed: Arc<dyn AdvisoryFeed> = match &options.advisory_feed {
            Some(feed) => feed.clone(),
            None => Arc::new(GithubAdvisoryFeed::new(secrets.clone())),
        };
        let scan_settings = ScanSettingsService::new(ctx.clone(), secrets.clone());
        let vulnerability_scans = VulnerabilityScanService::new(
            ctx.clone(),
            advisory_feed,
            services.inventory.clone(),
            services.advisories.clone(),
            services.exposures.clone(),
            scan_settings.clone(),
        );
        let api = ApiState::new(services.clone(), stream.clone(), config.user_home.clone(), secrets.clone(), jira_issues, vulnerability_scans.clone(), scan_settings);
        let mcp = rekall_mcp::McpController::new(rekall_mcp::tool_registry::all(&services));

        let claude_config = &config.claude;
        let login_shell = LoginShellEnvironment::new(claude_config.shell.as_deref(), claude_config.shell_environment_timeout);
        let cli = ClaudeCli::new(claude_config.cli_path.as_deref(), claude_config.home.clone(), Arc::new(login_shell.clone()));
        let broker = Arc::new(CredentialBroker::new(
            secrets.clone(),
            ConnectionAccessService::new(ctx.clone()),
            format!("http://127.0.0.1:{served_port}"),
            config.home.join(CREDENTIAL_SESSIONS_FOLDER),
        ));
        let terminals = PtyTerminalManager::new(
            services.terminal_launch.clone(),
            cli,
            services.steps.clone(),
            services.review.clone(),
            broker.clone(),
            claude_config,
        );
        let usage: Arc<dyn UsageReader> = match &options.usage {
            Some(usage) => usage.clone(),
            None => Arc::new(ClaudeUsageService::new(
                Arc::new(ClaudeCredentials::new(claude_config.home.clone())),
                &claude_config.usage_url,
                clock.clone(),
            )),
        };
        let queue = RunQueueService::new(ctx.clone());
        let runner = RunQueueRunner::start(
            queue.clone(),
            terminals.clone(),
            usage.clone(),
            services.task_work.clone(),
            services.steps.clone(),
            &events,
            clock.clone(),
            claude_config.run_queue_tick,
            claude_config.run_queue_settle_grace,
        );
        let transcripts = TranscriptScanner::new(
            TranscriptFolder::new(claude_config.transcripts_dir.clone()),
            services.token_usage.clone(),
        );
        let resources = ClaudeResourceService::new(claude_config.claude_home.clone());
        let skill_groups = SkillGroupService::new(ctx.clone());
        let claude = ClaudeState { terminals: terminals.clone(), usage: usage.clone(), queue, runner: runner.clone(), resources, skill_groups, broker };

        let location = database.path.as_deref().and_then(DatabaseLocation::of);
        let backups = DatabaseBackupService::new(
            database.conn.clone(),
            location,
            config.backup.enabled,
            config.backup.interval_hours,
            config.backup.keep,
            clock,
        );
        let restorer = DatabaseRestoreService::new(backups.clone(), restarter.clone());
        let settings = SettingsState {
            store: DatabaseRegistryStore::new(&config.home),
            user_home: config.user_home.clone(),
            restarter,
        };
        let installer = ClaudeCodeInstaller::new(config.user_home.clone(), config.mcp_endpoint(served_port));

        let release_feed: Arc<dyn ReleaseFeed> = match &options.release_feed {
            Some(feed) => feed.clone(),
            None => Arc::new(GithubReleaseFeed::new(&config.update_check.url)),
        };
        let update_checker = Arc::new(UpdateChecker::new(RUNNING_VERSION, config.update_check.enabled, release_feed));
        let connection_probe: Arc<dyn ConnectionProbe> = match &options.connection_probe {
            Some(probe) => probe.clone(),
            None => Arc::new(HttpConnectionProbe::new()),
        };
        let connection_checks = ConnectionCheckService::new(ctx.clone(), secrets.clone(), connection_probe);

        let routes = Router::new()
            .merge(rekall_api::router(api))
            .merge(rekall_mcp::router(mcp))
            .merge(rekall_infra::router(claude))
            .merge(settings::routes(settings))
            .merge(installer::routes(installer))
            .merge(backup::routes(backups.clone(), restorer))
            .merge(actuator::routes(database.conn.clone()))
            .merge(version::routes(update_checker))
            .merge(probes::routes(connection_checks))
            .merge(spa::routes(Assets::new(config.ui_dist.clone())))
            .method_not_allowed_fallback(|| async { framework(StatusCode::METHOD_NOT_ALLOWED) });
        // The routes sit behind a fallback so the layers below see each response whole.
        let router = Router::new()
            .fallback_service(routes)
            .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
            .layer(middleware::from_fn_with_state(LocalAccess::new(config.remote_access), security::guard))
            .layer(middleware::from_fn(render_errors));

        Self {
            database,
            services,
            status,
            router,
            stream,
            terminals,
            usage,
            runner,
            backups,
            login_shell,
            sweep_minutes: claude_config.sweep_minutes,
            idle_sweep: Mutex::new(None),
            transcripts,
            transcript_scan_every: claude_config.transcript_scan_every,
            transcript_scan: Mutex::new(None),
            vulnerability_scans,
            vulnerability_scan: Mutex::new(None),
        }
    }

    pub fn router(&self) -> Router {
        self.router.clone()
    }

    /// `ApplicationReadyEvent`.
    pub fn ready(&self) {
        self.login_shell.warm();
        self.terminals.start_reaper(self.sweep_minutes);
        self.backups.start_schedule();
        self.start_idle_sweep();
        *self.transcript_scan.lock().expect("never poisoned") = Some(self.transcripts.start(self.transcript_scan_every));
        self.start_vulnerability_scans();
    }

    fn start_vulnerability_scans(&self) {
        let scans = self.vulnerability_scans.clone();
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + VULNERABILITY_SCAN_AFTER, VULNERABILITY_SCAN_EVERY);
            loop {
                interval.tick().await;
                if let Err(failed) = scans.scan().await {
                    warn!(%failed, "The scheduled vulnerability scan failed");
                }
            }
        });
        *self.vulnerability_scan.lock().expect("never poisoned") = Some(handle);
    }

    fn start_idle_sweep(&self) {
        let time_entries = self.services.time_entries.clone();
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + IDLE_SWEEP_EVERY, IDLE_SWEEP_EVERY);
            loop {
                interval.tick().await;
                if let Err(failed) = time_entries.stop_idle(IDLE_AFTER).await {
                    warn!(%failed, "Stopping idle timers failed");
                }
            }
        });
        *self.idle_sweep.lock().expect("never poisoned") = Some(handle);
    }

    /// `ContextClosedEvent`: let go of everything that could hold the web server's shutdown.
    pub async fn close_hooks(&self) {
        self.runner.shutdown();
        self.usage.release_on_shutdown();
        self.stream.release_on_shutdown();
        self.terminals.shutdown().await;
        self.backups.stop_schedule();
        if let Some(sweep) = self.idle_sweep.lock().expect("never poisoned").take() {
            sweep.abort();
        }
        if let Some(scan) = self.transcript_scan.lock().expect("never poisoned").take() {
            scan.abort();
        }
        if let Some(scan) = self.vulnerability_scan.lock().expect("never poisoned").take() {
            scan.abort();
        }
        if let Err(failed) = self.services.time_entries.stop_all_on_shutdown().await {
            warn!(%failed, "Stopping open timers on shutdown failed");
        }
    }
}
