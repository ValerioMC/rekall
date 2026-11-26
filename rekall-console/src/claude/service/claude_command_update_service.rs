use std::sync::{Arc, Mutex};

use rekall_core::claude::ClaudeCodeSetup;
use tracing::{info, warn};

use crate::shared::blocking;
use crate::claude::view::command_update_state::CommandUpdateState;

/// Replaces the installed `/rk` command files with this build's when they are older, and keeps
/// where that stands for the console. Registrations and folder setups are left for Repair.
#[derive(Clone)]
pub struct ClaudeCommandUpdateService {
    setup: Arc<dyn ClaudeCodeSetup>,
    state: Arc<Mutex<CommandUpdateState>>,
}

impl ClaudeCommandUpdateService {
    pub fn new(setup: Arc<dyn ClaudeCodeSetup>) -> Self {
        Self { setup, state: Arc::new(Mutex::new(CommandUpdateState::Idle)) }
    }

    pub fn state(&self) -> CommandUpdateState {
        self.state.lock().expect("never poisoned").clone()
    }

    pub async fn run_if_outdated(&self) {
        let setup = self.setup.clone();
        let Ok(installation) = blocking(move || Ok(setup.status())).await else {
            return;
        };
        if !installation.needs_command_update() {
            return;
        }
        self.remember(CommandUpdateState::Updating);
        let setup = self.setup.clone();
        match blocking(move || setup.update_commands()).await {
            Ok(_) => {
                info!(files = %installation.outdated_files.join(", "), "Updated the installed /rk commands");
                self.remember(CommandUpdateState::Updated);
            }
            Err(failure) => {
                warn!(%failure, files = %installation.outdated_files.join(", "), "Could not update the installed /rk commands");
                self.remember(CommandUpdateState::Failed { reason: failure.to_string() });
            }
        }
    }

    fn remember(&self, state: CommandUpdateState) {
        *self.state.lock().expect("never poisoned") = state;
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/claude/service/claude_command_update_service_tests.rs"]
mod tests;
