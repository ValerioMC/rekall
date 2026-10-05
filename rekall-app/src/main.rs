//! `rekall-server`: the application on http://localhost:47355 (or `SERVER_PORT`), until SIGTERM
//! or Ctrl-C.

use std::process::ExitCode;

use rekall_app::{AppConfig, StartOptions};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,sea_orm=warn,sea_orm_migration=warn")),
        )
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "rekall-server [--server.port=N] [--rekall.home=DIR] [--<property>=<value> ...]\n\
             \n\
             Properties can also be set as environment variables (SERVER_PORT, REKALL_HOME, REKALL_DB_URL, ...)."
        );
        return ExitCode::SUCCESS;
    }
    let config = AppConfig::from_process(&args);

    let running = match rekall_app::start(config, StartOptions::default()).await {
        Ok(running) => running,
        Err(failed) => {
            error!("Rekall could not start: {failed}");
            return ExitCode::FAILURE;
        }
    };
    shutdown_signal().await;
    info!("Shutting down");
    if tokio::time::timeout(rekall_app::server::STOP_WINDOW, running.stop()).await.is_err() {
        error!("Shutdown did not finish in time");
    }
    ExitCode::SUCCESS
}

async fn shutdown_signal() {
    let interrupt = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("signal handler");
        tokio::select! {
            _ = interrupt => {}
            _ = terminate.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = interrupt.await;
    }
}
