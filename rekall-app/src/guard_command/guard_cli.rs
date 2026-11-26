use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;

use rekall_core::claude::SessionProvider;
use rekall_core::guard::{GuardScope, GUARD_PORT_ENV, GUARD_SCOPE_ENV};
use tracing::warn;

use super::context_source::LocalRekall;
use super::guard_mode::GuardMode;
use super::guard_outcome::EXIT_ALLOW;
use super::guard_runner::run;
use crate::AppConfig;

const LOG_FILE: &str = "guard.log";
const EXIT_USAGE: u8 = 1;

pub async fn execute(args: &[String]) -> u8 {
    let config = AppConfig::from_process(&[]);
    let _logging = log_to_file_in(&config.home);
    let (provider, event) = match args {
        [event] => (SessionProvider::Claude, event),
        [provider, event] if provider == "codex" => (SessionProvider::Codex, event),
        _ => return usage("guard needs an event: before-tool or before-stop"),
    };
    let mode = match event.parse::<GuardMode>() {
        Ok(mode) => mode,
        Err(failure) => return usage(&failure),
    };
    let mut input_text = String::new();
    if let Err(failure) = std::io::stdin().read_to_string(&mut input_text) {
        warn!(%failure, "The guard could not read its input and let the action through");
        return EXIT_ALLOW;
    }
    let port = guard_port_from_process().unwrap_or(config.port);
    let outcome = run(provider, mode, &input_text, scope_from_process(), &LocalRekall::new(config.local_url(port))).await;
    if let Some(message) = outcome.stdout {
        println!("{message}");
    }
    if let Some(message) = outcome.stderr {
        eprintln!("{message}");
    }
    outcome.exit_code
}

fn guard_port_from_process() -> Option<u16> {
    match std::env::var(GUARD_PORT_ENV) {
        Ok(raw) => match raw.parse() {
            Ok(port) => Some(port),
            Err(failure) => {
                warn!(%failure, "The guard received an invalid server port");
                None
            }
        },
        Err(std::env::VarError::NotPresent) => None,
        Err(failure) => {
            warn!(%failure, "The guard could not read its server port");
            None
        }
    }
}

fn scope_from_process() -> Option<GuardScope> {
    match std::env::var(GUARD_SCOPE_ENV) {
        Ok(raw) => match raw.parse() {
            Ok(scope) => Some(scope),
            Err(failure) => {
                warn!(%failure, "The guard received an invalid launch scope");
                Some(GuardScope::WithoutStep)
            }
        },
        Err(std::env::VarError::NotPresent) => None,
        Err(failure) => {
            warn!(%failure, "The guard could not read its launch scope");
            Some(GuardScope::WithoutStep)
        }
    }
}

fn usage(message: &str) -> u8 {
    eprintln!("rekall-server guard [codex] before-tool|before-stop: {message}");
    EXIT_USAGE
}

fn log_to_file_in(home: &Path) -> tracing::subscriber::DefaultGuard {
    let file = std::fs::create_dir_all(home).and_then(|()| OpenOptions::new().create(true).append(true).open(home.join(LOG_FILE)));
    let subscriber = tracing_subscriber::fmt().with_ansi(false);
    match file {
        Ok(file) => tracing::subscriber::set_default(subscriber.with_writer(Mutex::new(file)).finish()),
        Err(_) => tracing::subscriber::set_default(subscriber.with_writer(std::io::stderr).finish()),
    }
}
