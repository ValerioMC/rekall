use std::path::Path;
use std::time::Duration;

use async_trait::async_trait;
use rekall_kernel::Result;

use crate::check::CheckRun;

/// Runs a shell command in a folder and reports how it ended. It decides nothing: which command,
/// where and for how long are the caller's. At the deadline it ends the command and its children
/// and answers `TimedOut`; an `Err` is a command that could not be started at all.
#[async_trait]
pub trait CheckRunner: Send + Sync {
    async fn run(&self, folder: &Path, command: &str, timeout: Duration) -> Result<CheckRun>;
}
