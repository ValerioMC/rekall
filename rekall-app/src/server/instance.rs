use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::{middleware, Router};
use rekall_api::error::{framework, render_errors};
use rekall_api::{ApiState, StepEventStream};
use rekall_claude::cli::ClaudeCli;
use rekall_claude::credentials::ClaudeCredentials;
use rekall_claude::login_shell::LoginShellEnvironment;
use rekall_claude::pty::PtyTerminalManager;
use rekall_claude::queue::{RunQueueRunner, RunQueueService};
use rekall_claude::usage::{ClaudeUsageService, UsageReader};
use rekall_claude::ClaudeState;
use rekall_repository::Database;
use rekall_service::timeentry::IDLE_AFTER;
use rekall_service::{Ctx, EventBus, Services};
use tracing::warn;

use crate::backup::{self, DatabaseBackupService, DatabaseLocation, DatabaseRestoreService};
use crate::bootstrap::installer::{self, ClaudeCodeInstaller};
use crate::bootstrap::location::SetupStatus;
use crate::bootstrap::registry::DatabaseRegistryStore;
use crate::bootstrap::settings::{self, SettingsState};
use crate::config::AppConfig;
use crate::restart::Restarter;
use crate::security::{self, LocalAccess};
use crate::spa::{self, Assets};
use crate::actuator;

use super::StartOptions;

/// How often open timers are checked against [`IDLE_AFTER`].
const IDLE_SWEEP_EVERY: Duration = Duration::from_secs(60);

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

        let stream = Arc::new(StepEventStream::new(events.clone()));
        let api = ApiState::new(services.clone(), stream.clone(), config.user_home.clone());
        let mcp = rekall_mcp::McpController::new(rekall_mcp::tool::all(&services));

        let claude_config = &config.claude;
        let login_shell = LoginShellEnvironment::new(claude_config.shell.as_deref(), claude_config.shell_environment_timeout);
        let cli = ClaudeCli::new(claude_config.cli_path.as_deref(), claude_config.home.clone(), Arc::new(login_shell.clone()));
        let terminals = PtyTerminalManager::new(
            services.terminal_launch.clone(),
            cli,
            services.steps.clone(),
            services.review.clone(),
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
        let claude = ClaudeState { api: api.clone(), terminals: terminals.clone(), usage: usage.clone(), queue, runner: runner.clone() };

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

        let routes = Router::new()
            .merge(rekall_api::router(api))
            .merge(rekall_mcp::router(mcp))
            .merge(rekall_claude::router(claude))
            .merge(settings::routes(settings))
            .merge(installer::routes(installer))
            .merge(backup::routes(backups.clone(), restorer))
            .merge(actuator::routes(database.conn.clone()))
            .merge(spa::routes(Assets::new(config.ui_dist.clone())))
            .method_not_allowed_fallback(|| async { framework(StatusCode::METHOD_NOT_ALLOWED) });
        // The routes sit behind a fallback so the layers below see each response whole, the
        // `Allow` header Axum adds to a 405 included.
        let router = Router::new()
            .fallback_service(routes)
            .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
            .layer(middleware::from_fn(crate::methods::spring_methods))
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
    }

    fn start_idle_sweep(&self) {
        let time_entries = self.services.time_entries.clone();
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + IDLE_SWEEP_EVERY, IDLE_SWEEP_EVERY);
            loop {
                interval.tick().await;
                if let Err(failed) = time_entries.stop_idle(IDLE_AFTER).await {
                    warn!("Stopping idle timers: {failed}");
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
        if let Err(failed) = self.services.time_entries.stop_all_on_shutdown().await {
            warn!("Stopping open timers on shutdown: {failed}");
        }
    }
}
