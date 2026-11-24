use std::sync::atomic::{AtomicUsize, Ordering};

use rekall_core::claude::Installation;
use rekall_kernel::{RekallError, Result};

use super::*;

/// A setup reporting one status, counting the rewrites asked of it and failing them on request.
struct FakeSetup {
    status: &'static str,
    outdated_files: Vec<String>,
    update_fails: bool,
    updates: AtomicUsize,
}

impl FakeSetup {
    fn reporting(status: &'static str, outdated_files: &[&str]) -> Self {
        Self {
            status,
            outdated_files: outdated_files.iter().map(|file| file.to_string()).collect(),
            update_fails: false,
            updates: AtomicUsize::new(0),
        }
    }

    fn failing(mut self) -> Self {
        self.update_fails = true;
        self
    }

    fn installation(&self) -> Installation {
        Installation {
            status: self.status,
            endpoint: "http://127.0.0.1:47355/mcp".into(),
            registered_url: None,
            folder_scoped: Vec::new(),
            command_installed: self.outdated_files.is_empty(),
            outdated_files: self.outdated_files.clone(),
            cli_path: None,
            manual_command: "claude mcp add rekall".into(),
        }
    }
}

impl ClaudeCodeSetup for FakeSetup {
    fn status(&self) -> Installation {
        self.installation()
    }

    fn install(&self) -> Result<Installation> {
        unreachable!("the startup update never registers")
    }

    fn update_commands(&self) -> Result<Installation> {
        self.updates.fetch_add(1, Ordering::SeqCst);
        if self.update_fails {
            return Err(RekallError::conflict("Could not write ~/.claude/commands/rk.md"));
        }
        Ok(self.installation())
    }
}

async fn run(setup: &Arc<FakeSetup>) -> ClaudeCommandUpdateService {
    let service = ClaudeCommandUpdateService::new(setup.clone());
    service.run_if_outdated().await;
    service
}

#[tokio::test]
async fn outdated_command_files_are_replaced_and_the_state_ends_updated() {
    let setup = Arc::new(FakeSetup::reporting("OUTDATED", &["commands/rk.md"]));

    let service = run(&setup).await;

    assert_eq!(setup.updates.load(Ordering::SeqCst), 1);
    assert_eq!(service.state(), CommandUpdateState::Updated);
}

#[tokio::test]
async fn up_to_date_files_are_not_written_and_the_state_stays_idle() {
    let setup = Arc::new(FakeSetup::reporting("CONNECTED", &[]));

    let service = run(&setup).await;

    assert_eq!(setup.updates.load(Ordering::SeqCst), 0);
    assert_eq!(service.state(), CommandUpdateState::Idle);
}

#[tokio::test]
async fn a_write_error_ends_failed_with_its_reason() {
    let setup = Arc::new(FakeSetup::reporting("OUTDATED", &["commands/rk.md"]).failing());

    let service = run(&setup).await;

    assert_eq!(setup.updates.load(Ordering::SeqCst), 1);
    assert_eq!(service.state(), CommandUpdateState::Failed { reason: "Could not write ~/.claude/commands/rk.md".into() });
}

#[tokio::test]
async fn a_missing_cli_or_a_missing_registration_writes_nothing() {
    for status in ["CLI_MISSING", "NOT_CONNECTED"] {
        let setup = Arc::new(FakeSetup::reporting(status, &["commands/rk.md"]));

        let service = run(&setup).await;

        assert_eq!(setup.updates.load(Ordering::SeqCst), 0, "{status}");
        assert_eq!(service.state(), CommandUpdateState::Idle, "{status}");
    }
}

#[tokio::test]
async fn an_outdated_registration_with_current_files_writes_nothing() {
    let setup = Arc::new(FakeSetup::reporting("OUTDATED", &[]));

    let service = run(&setup).await;

    assert_eq!(setup.updates.load(Ordering::SeqCst), 0);
    assert_eq!(service.state(), CommandUpdateState::Idle);
}
