//! `rekall-server`: the application on http://localhost:47355 (or `SERVER_PORT`), until SIGTERM
//! or Ctrl-C. `--migrate-from-h2 <folder>` first imports the Java build's H2 database there.

use std::path::PathBuf;
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

    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let migrate = take_option(&mut args, "--migrate-from-h2");
    let h2_jar = take_option(&mut args, "--h2-jar").map(PathBuf::from);
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "rekall-server [--server.port=N] [--rekall.home=DIR] [--<property>=<value> ...]\n\
             rekall-server --migrate-from-h2 <folder|rekall.mv.db> [--h2-jar <h2.jar>]\n\n\
             Properties can also be set as environment variables (SERVER_PORT, REKALL_HOME, REKALL_DB_URL, ...)."
        );
        return ExitCode::SUCCESS;
    }
    let config = AppConfig::from_process(&args);

    if let Some(source) = migrate {
        match rekall_app::migrate_from_h2(&config, std::path::Path::new(&source), h2_jar).await {
            Ok(report) => {
                info!("Imported {} rows over {} changesets from {source}", report.total_rows(), report.changesets_in_source);
                for (table, rows) in &report.rows {
                    info!("  {table}: {rows}");
                }
            }
            Err(failed) => {
                error!("The import failed: {failed}");
                return ExitCode::FAILURE;
            }
        }
    }

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

/// `--name value` or `--name=value`, removed from the arguments.
fn take_option(args: &mut Vec<String>, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    if let Some(index) = args.iter().position(|a| a.starts_with(&prefix)) {
        return Some(args.remove(index)[prefix.len()..].to_string());
    }
    let index = args.iter().position(|a| a == name)?;
    args.remove(index);
    (index < args.len()).then(|| args.remove(index))
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
