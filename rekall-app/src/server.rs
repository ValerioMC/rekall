//! The application: every module's router over one set of services on one database, served on
//! one port, and the supervisor that brings it down and back up in the same process when the
//! database changes (`ApplicationRestarter`).
//!
//! Closing follows Spring's order: the closing hooks run first (the run queue stops deciding, the
//! usage fetch and the event stream let go, every terminal is killed, the backup schedule stops),
//! then the web server stops taking requests and waits a bounded time for the ones in flight,
//! then the database pool closes.

use std::net::{Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::sync::Arc;
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
use rekall_service::{Clock, Ctx, EventBus, Services};
use tokio::sync::{oneshot, watch};
use tracing::{error, info, warn};

use crate::backup::{self, DatabaseBackupService, DatabaseLocation, DatabaseRestoreService};
use crate::bootstrap::installer::{self, ClaudeCodeInstaller};
use crate::bootstrap::location::{self, SetupStatus};
use crate::bootstrap::registry::DatabaseRegistryStore;
use crate::bootstrap::settings::{self, SettingsState};
use crate::config::{AppConfig, DatabaseOverride};
use crate::restart::{self, Restarter, RESTART_DELAY};
use crate::security::{self, LocalAccess};
use crate::spa::{self, Assets};
use crate::{actuator, h2};

/// What a caller can swap in, as the tests' `@MockitoBean`s did.
#[derive(Clone, Default)]
pub struct StartOptions {
    /// Stands in for the Claude usage service.
    pub usage: Option<Arc<dyn UsageReader>>,
    /// Stands in for the system clock.
    pub clock: Option<Clock>,
    /// Hand the instance a restarter nothing listens to, as a Spring test context had: a
    /// database switch is recorded but the application stays on the database it has.
    pub no_restart: bool,
}

/// One start of the application on one database.
pub struct Instance {
    pub database: Database,
    pub services: Services,
    pub status: SetupStatus,
    router: Router,
    stream: Arc<StepEventStream>,
    terminals: Arc<PtyTerminalManager>,
    usage: Arc<dyn UsageReader>,
    runner: RunQueueRunner,
    backups: DatabaseBackupService,
    login_shell: Arc<LoginShellEnvironment>,
    sweep_minutes: u64,
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
        let api = ApiState::new(services.clone(), stream.clone());
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

        let router = Router::new()
            .merge(rekall_api::router(api))
            .merge(rekall_mcp::router(mcp))
            .merge(rekall_claude::router(claude))
            .merge(settings::routes(settings))
            .merge(installer::routes(installer))
            .merge(backup::routes(backups.clone(), restorer))
            .merge(actuator::routes(database.conn.clone()))
            .merge(spa::routes(Assets::new(config.ui_dist.clone())))
            .method_not_allowed_fallback(|| async { framework(StatusCode::METHOD_NOT_ALLOWED) })
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
    }

    /// `ContextClosedEvent`: let go of everything that could hold the web server's shutdown.
    pub async fn close_hooks(&self) {
        self.runner.shutdown();
        self.usage.release_on_shutdown();
        self.stream.release_on_shutdown();
        self.terminals.shutdown().await;
        self.backups.stop_schedule();
    }
}

/// A running server: the port it answers on, the services of the instance currently up, and the
/// way to stop it.
pub struct Running {
    pub port: u16,
    current: watch::Receiver<Option<(u64, Services)>>,
    stop: Option<oneshot::Sender<()>>,
    supervisor: Option<tokio::task::JoinHandle<()>>,
}

impl Running {
    /// The services of the instance up right now; `None` in the gap of a restart.
    pub fn services(&self) -> Option<Services> {
        self.current.borrow().as_ref().map(|(_, services)| services.clone())
    }

    /// Which start of the application is up: 1 for the first, one more after every restart.
    pub fn generation(&self) -> u64 {
        self.current.borrow().as_ref().map(|(generation, _)| *generation).unwrap_or(0)
    }

    /// Wait until a start later than `generation` is up (a restart has gone round).
    pub async fn instance_after(&mut self, generation: u64) -> Option<Services> {
        let found = self
            .current
            .wait_for(|current| current.as_ref().is_some_and(|(g, _)| *g > generation))
            .await
            .ok()?;
        found.as_ref().map(|(_, services)| services.clone())
    }

    /// Close the application and wait for it.
    pub async fn stop(mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(supervisor) = self.supervisor.take() {
            let _ = supervisor.await;
        }
    }
}

/// Bind the port and start the supervisor. The listening socket is kept across restarts, so a
/// request that arrives in the gap waits for the next instance instead of being refused.
pub async fn start(config: AppConfig, options: StartOptions) -> Result<Running, String> {
    let listener = bind(config.port)?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let (current_sender, current) = watch::channel(None);
    let (stop, stop_signal) = oneshot::channel();
    let (first_up, first_up_signal) = oneshot::channel();
    let supervisor = tokio::spawn(supervise(config, options, listener, port, current_sender, stop_signal, first_up));
    first_up_signal.await.map_err(|_| "The application did not start.".to_string())??;
    info!("Rekall is listening on http://localhost:{port}");
    Ok(Running { port, current, stop: Some(stop), supervisor: Some(supervisor) })
}

async fn supervise(
    config: AppConfig,
    options: StartOptions,
    listener: std::net::TcpListener,
    port: u16,
    current: watch::Sender<Option<(u64, Services)>>,
    mut stop_signal: oneshot::Receiver<()>,
    first_up: oneshot::Sender<Result<(), String>>,
) {
    let mut first_up = Some(first_up);
    let mut generation = 0;
    loop {
        generation += 1;
        let (instance, mut restart_requests) = match open_instance(&config, &options, port).await {
            Ok(opened) => opened,
            Err(failed) => {
                error!("Could not start: {failed}");
                if let Some(first_up) = first_up.take() {
                    let _ = first_up.send(Err(failed));
                }
                return;
            }
        };
        let tokio_listener = match listener.try_clone().and_then(tokio::net::TcpListener::from_std) {
            Ok(tokio_listener) => tokio_listener,
            Err(failed) => {
                error!("Could not listen on port {port}: {failed}");
                if let Some(first_up) = first_up.take() {
                    let _ = first_up.send(Err(failed.to_string()));
                }
                return;
            }
        };
        let (serving_stop, serving_stop_signal) = oneshot::channel::<()>();
        let app = instance.router().into_make_service_with_connect_info::<SocketAddr>();
        let server = tokio::spawn(async move {
            let _ = axum::serve(tokio_listener, app)
                .with_graceful_shutdown(async move {
                    let _ = serving_stop_signal.await;
                })
                .await;
        });
        instance.ready();
        let _ = current.send(Some((generation, instance.services.clone())));
        if let Some(first_up) = first_up.take() {
            let _ = first_up.send(Ok(()));
        }

        // A closed restart channel (a restarter nothing can use) is not a request to stop.
        let request = tokio::select! {
            _ = &mut stop_signal => None,
            Some(request) = restart_requests.recv() => Some(request),
        };
        if request.is_some() {
            restart::announce();
            tokio::time::sleep(RESTART_DELAY).await;
        }
        let (before_close, after_close) = match request {
            Some(request) => (Some(request.before_close), Some(request.after_close)),
            None => (None, None),
        };
        if let Some(hook) = before_close {
            restart::run_hook("before close", hook);
        }

        let _ = current.send(None);
        instance.close_hooks().await;
        let _ = serving_stop.send(());
        if tokio::time::timeout(config.shutdown_timeout, server).await.is_err() {
            warn!("Requests were still in flight after {:?}; closing anyway", config.shutdown_timeout);
        }
        let Instance { database, router, .. } = instance;
        drop(router);
        if let Err(failed) = database.close().await {
            warn!("Closing the database: {failed}");
        }

        match after_close {
            Some(hook) => restart::run_hook("after close", hook),
            None => {
                info!("Rekall stopped");
                return;
            }
        }
    }
}

async fn open_instance(
    config: &AppConfig,
    options: &StartOptions,
    port: u16,
) -> Result<(Instance, tokio::sync::mpsc::UnboundedReceiver<restart::RestartRequest>), String> {
    let resolved = location::resolve(config)?;
    let database = open_database(&resolved.database).await?;
    let (restarter, requests) = Restarter::channel();
    let restarter = if options.no_restart { Restarter::disabled() } else { restarter };
    Ok((Instance::build(config, database, resolved.status, restarter, port, options), requests))
}

/// A file that is not there yet but has the Java build's H2 database beside it is imported from
/// it first. An import that fails leaves the application on an in-memory database, with the
/// reason in the log, rather than an empty file over the person's data.
pub async fn open_database(target: &DatabaseOverride) -> Result<Database, String> {
    match target {
        DatabaseOverride::Memory => rekall_repository::open_in_memory().await.map_err(|e| e.to_string()),
        DatabaseOverride::File(path) => {
            if !path.exists() {
                if let Some(legacy) = h2::legacy_file_beside(path) {
                    info!("{} holds the Java build's H2 database; importing it into {}", legacy.display(), path.display());
                    let (source, target) = (legacy.clone(), path.clone());
                    let imported = tokio::task::spawn_blocking(move || h2::Importer::locate()?.import_blocking(&source, &target))
                        .await
                        .map_err(|e| e.to_string())
                        .and_then(|r| r);
                    if let Err(failed) = imported {
                        error!(
                            "Could not import {}: {failed}. Running on an empty in-memory database instead; \
                             run `rekall-server --migrate-from-h2 {}` to import it by hand.",
                            legacy.display(),
                            legacy.display()
                        );
                        return rekall_repository::open_in_memory().await.map_err(|e| e.to_string());
                    }
                }
            }
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            rekall_repository::open(path).await.map_err(|e| format!("Could not open {}: {e}", path.display()))
        }
    }
}

/// Every interface, IPv6 and IPv4 on one socket where the system allows it, as Tomcat did.
fn bind(port: u16) -> Result<std::net::TcpListener, String> {
    use socket2::{Domain, Protocol, Socket, Type};
    let dual = (|| -> std::io::Result<Socket> {
        let socket = Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP))?;
        socket.set_only_v6(false)?;
        socket.set_reuse_address(true)?;
        socket.bind(&SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0)).into())?;
        Ok(socket)
    })();
    let socket = match dual {
        Ok(socket) => socket,
        Err(_) => {
            let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).map_err(|e| e.to_string())?;
            socket.set_reuse_address(true).map_err(|e| e.to_string())?;
            socket
                .bind(&SocketAddr::V4(SocketAddrV4::new(std::net::Ipv4Addr::UNSPECIFIED, port)).into())
                .map_err(|e| format!("Port {port} is taken: {e}"))?;
            socket
        }
    };
    socket.listen(1024).map_err(|e| e.to_string())?;
    socket.set_nonblocking(true).map_err(|e| e.to_string())?;
    Ok(socket.into())
}

/// How long `stop()` waits at most, the launcher's window before it hard-kills the process.
pub const STOP_WINDOW: Duration = Duration::from_secs(10);
