use std::path::Path;

use rekall_common::{jstr, RekallError, Result};

use super::{PendingChange, display_path, run};
use super::git_command::COMMAND_TIMEOUT;

/// The one place that writes to a repository: stages everything and commits it, as whoever git
/// is configured to be in that folder.
#[derive(Clone, Copy, Debug, Default)]
pub struct GitCommitter;

impl GitCommitter {
    /// Every change `git add -A` would pick up, untracked files listed one by one.
    pub async fn pending_changes(&self, folder: &Path) -> Result<Vec<PendingChange>> {
        let status = run(folder, &["status", "--porcelain", "-uall"], None, COMMAND_TIMEOUT).await?;
        if !status.ok() {
            return Err(RekallError::illegal(format!(
                "Could not read the working tree in {}: {}",
                display_path(folder),
                status.error()
            )));
        }
        status
            .stdout
            .split('\n')
            .filter(|line| !jstr::is_blank(line))
            .map(PendingChange::parse)
            .collect()
    }

    /// Stages everything, commits it with `message`, and hands back the new tip's hash.
    pub async fn commit_all(&self, folder: &Path, message: &str) -> Result<String> {
        let shown = display_path(folder);
        let staged = run(folder, &["add", "-A"], None, COMMAND_TIMEOUT).await?;
        if !staged.ok() {
            return Err(RekallError::illegal(format!("Could not stage the changes in {shown}: {}", staged.error())));
        }
        let committed = run(folder, &["commit", "-q", "-F", "-"], Some(message), COMMAND_TIMEOUT).await?;
        if !committed.ok() {
            let reason = if jstr::is_blank(committed.error()) { committed.output() } else { committed.error() };
            return Err(RekallError::illegal(format!("git commit failed in {shown}: {reason}")));
        }
        let tip = run(folder, &["rev-parse", "HEAD"], None, COMMAND_TIMEOUT).await?;
        if !tip.ok() || jstr::is_blank(tip.output()) {
            return Err(RekallError::illegal(format!("Committed, but could not read the new tip in {shown}.")));
        }
        Ok(tip.output().to_string())
    }
}
