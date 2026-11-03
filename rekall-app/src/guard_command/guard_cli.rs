use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;

use tracing::warn;

use super::context_source::LocalRekall;
use super::guard_mode::GuardMode;
use super::guard_outcome::EXIT_ALLOW;
use super::guard_runner::run;
use crate::AppConfig;

const LOG_FILE: &str = "guard.log";
const EXIT_USAGE: u8 = 1;

/// The entry point of `guard <event>`; `args` are the process arguments after `guard`. It writes
/// the reason to standard error itself, since Claude Code reads it there.
pub async fn execute(args: &[String]) -> u8 {
    let config = AppConfig::from_process(&[]);
    let _logging = log_to_file_in(&config.home);
    let mode = match args.first().map(|event| event.parse::<GuardMode>()) {
        Some(Ok(mode)) => mode,
        Some(Err(failure)) => return usage(&failure),
        None => return usage("guard needs an event: before-tool or before-stop"),
    };
    let mut input_text = String::new();
    if let Err(failure) = std::io::stdin().read_to_string(&mut input_text) {
        warn!(%failure, "The guard could not read its input and let the action through");
        return EXIT_ALLOW;
    }
    let outcome = run(mode, &input_text, &LocalRekall::new(config.local_url(config.port))).await;
    if let Some(reason) = &outcome.reason {
        eprintln!("{reason}");
    }
    outcome.exit_code
}

fn usage(message: &str) -> u8 {
    eprintln!("rekall-server guard before-tool|before-stop: {message}");
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
