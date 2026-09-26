use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use rekall_common::{RekallError, Result};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use super::GitResult;

/// The folder as `Path.of(folder)` printed it: no trailing separator, no doubled ones.
pub fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    let mut out = String::with_capacity(text.len());
    let mut previous_slash = false;
    for c in text.chars() {
        if c == '/' {
            if previous_slash {
                continue;
            }
            previous_slash = true;
        } else {
            previous_slash = false;
        }
        out.push(c);
    }
    while out.len() > 1 && out.ends_with('/') {
        out.pop();
    }
    out
}

/// `GitCommand.run`: `git -C <folder> <arguments>`, with `stdin` fed to it when given (how a
/// multi-line commit message travels), refused after `timeout`.
pub async fn run(folder: &Path, arguments: &[&str], stdin: Option<&str>, timeout: Duration) -> Result<GitResult> {
    let shown = display_path(folder);
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(folder)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command
        .spawn()
        .map_err(|e| RekallError::illegal(format!("Could not run git in {shown}: {e}")))?;
    if let Some(mut input) = child.stdin.take() {
        if let Some(text) = stdin {
            input
                .write_all(text.as_bytes())
                .await
                .map_err(|e| RekallError::illegal(format!("Could not read git's output in {shown}: {e}")))?;
        }
        drop(input);
    }
    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Err(_) => Err(RekallError::illegal(format!("git took too long to answer in {shown}."))),
        Ok(Err(e)) => Err(RekallError::illegal(format!("Could not read git's output in {shown}: {e}"))),
        Ok(Ok(output)) => Ok(GitResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }),
    }
}

pub(super) const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);
pub(super) const READ_TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(test)]
#[path = "git_command_tests.rs"]
mod tests;
