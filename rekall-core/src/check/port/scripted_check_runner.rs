use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use rekall_kernel::Result;

use crate::check::{CheckOutcome, CheckRun, CheckRunner};

/// The `CheckRunner` tests use: each command answers with the outcome and output it was given (a
/// command nobody scripted exits 0), and every run is recorded in order.
#[derive(Default)]
pub struct ScriptedCheckRunner {
    scripted: Mutex<HashMap<String, (CheckOutcome, String)>>,
    runs: Mutex<Vec<(PathBuf, String, Duration)>>,
}

impl ScriptedCheckRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn answering(self, command: &str, outcome: CheckOutcome, output_tail: &str) -> Self {
        self.scripted.lock().expect("the lock is never poisoned").insert(command.to_string(), (outcome, output_tail.to_string()));
        self
    }

    /// The folder, command and timeout of every run so far.
    pub fn runs(&self) -> Vec<(PathBuf, String, Duration)> {
        self.runs.lock().expect("the lock is never poisoned").clone()
    }
}

#[async_trait]
impl CheckRunner for ScriptedCheckRunner {
    async fn run(&self, folder: &Path, command: &str, timeout: Duration) -> Result<CheckRun> {
        self.runs.lock().expect("the lock is never poisoned").push((folder.to_path_buf(), command.to_string(), timeout));
        let (outcome, output_tail) = self.scripted.lock().expect("the lock is never poisoned").get(command).cloned().unwrap_or((CheckOutcome::Exited(0), String::new()));
        Ok(CheckRun { outcome, output_tail, duration: Duration::from_millis(5) })
    }
}
