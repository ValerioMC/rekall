use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use rekall_core::check::{CheckOutcome, CheckRun, CheckRunner};
use rekall_kernel::{InternalFailure, RekallError, Result};
use tracing::warn;

use super::output_tail::output_tail;
use crate::login_shell::{kill_group, Environment, LoginShellEnvironment};

const POLL_INTERVAL: Duration = Duration::from_millis(20);
const OUTPUT_WINDOW_BYTES: usize = 64 * 1024;

/// Runs a check in the environment of the user's login shell, so `cargo` and `pnpm` are on the
/// `PATH`. Standard output and error share one pipe, and only its last lines are kept: the output
/// may hold secrets and is never logged.
pub struct ShellCheckRunner {
    environment: Arc<LoginShellEnvironment>,
}

impl ShellCheckRunner {
    pub fn new(environment: Arc<LoginShellEnvironment>) -> Self {
        Self { environment }
    }
}

#[async_trait]
impl CheckRunner for ShellCheckRunner {
    async fn run(&self, folder: &Path, command: &str, timeout: Duration) -> Result<CheckRun> {
        if !folder.is_dir() {
            return Err(RekallError::illegal(format!("The folder '{}' does not exist.", folder.display())));
        }
        let environment = self.environment.current().await;
        let folder = folder.to_path_buf();
        let command = command.to_string();
        tokio::task::spawn_blocking(move || run_to_the_end(&folder, &command, &environment, timeout))
            .await
            .map_err(|failed| RekallError::internal(InternalFailure::Background, failed))?
    }
}

fn run_to_the_end(folder: &Path, command: &str, environment: &Environment, timeout: Duration) -> Result<CheckRun> {
    let started = Instant::now();
    let (mut reader, writer) = std::io::pipe().map_err(io_failure)?;
    let mut shell = shell_command(command);
    shell
        .current_dir(folder)
        .env_clear()
        .envs(environment)
        .stdin(Stdio::null())
        .stdout(writer.try_clone().map_err(io_failure)?)
        .stderr(writer);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        shell.process_group(0);
    }
    let mut child = shell.spawn().map_err(io_failure)?;
    drop(shell);
    let tail = std::thread::spawn(move || read_last_bytes(&mut reader));
    let outcome = wait_until(&mut child, started + timeout);
    kill_group(&mut child);
    let output = tail.join().unwrap_or_else(|_| {
        warn!("The thread reading a check's output panicked");
        Vec::new()
    });
    Ok(CheckRun { outcome, output_tail: output_tail(&output), duration: started.elapsed() })
}

#[cfg(unix)]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("/bin/sh");
    shell.args(["-c", command]);
    shell
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("cmd");
    shell.args(["/C", command]);
    shell
}

fn wait_until(child: &mut std::process::Child, deadline: Instant) -> CheckOutcome {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return CheckOutcome::Exited(status.code().unwrap_or(-1)),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(POLL_INTERVAL),
            Ok(None) => return CheckOutcome::TimedOut,
            Err(failed) => {
                warn!(%failed, "Could not wait for a check's command");
                return CheckOutcome::Exited(-1);
            }
        }
    }
}

fn read_last_bytes(reader: &mut impl Read) -> Vec<u8> {
    let mut window: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 8 * 1024];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) | Err(_) => return window,
            Ok(count) => {
                window.extend_from_slice(&chunk[..count]);
                let excess = window.len().saturating_sub(OUTPUT_WINDOW_BYTES);
                window.drain(..excess);
            }
        }
    }
}

fn io_failure(failed: std::io::Error) -> RekallError {
    RekallError::internal(InternalFailure::Io, format!("Could not run a check's command: {failed}"))
}

#[cfg(test)]
#[path = "../../tests/unit/check/shell_check_runner_tests.rs"]
mod tests;
