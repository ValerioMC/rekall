use std::net::{Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::time::Duration;

use rekall_repository::Database;
use rekall_service::Services;
use tokio::sync::{oneshot, watch};
use tracing::{error, info, warn};

use crate::bootstrap::location;
use crate::config::{AppConfig, DatabaseOverride};
use crate::restart::{self, Restarter, RESTART_DELAY};
use crate::h2;

use super::{Instance, Running, StartOptions};

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
