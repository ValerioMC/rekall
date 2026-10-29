use std::sync::Arc;

use rekall_core::claude::{TerminalHost, TerminalOpening, TerminalTarget, TerminalView};
use rekall_kernel::{Id, RekallError, Result};

use crate::terminal::OpenTerminalCommand;

const CLOSED_FROM_THE_CONSOLE: &str = "Closed from the console.";

#[derive(Clone)]
pub struct TerminalService {
    host: Arc<dyn TerminalHost>,
}

impl TerminalService {
    pub fn new(host: Arc<dyn TerminalHost>) -> Self {
        Self { host }
    }

    pub fn list(&self) -> Vec<TerminalView> {
        self.host.list()
    }

    pub fn get(&self, id: Id) -> Result<TerminalView> {
        self.host.get(id).ok_or_else(|| no_terminal(id))
    }

    /// A project alone has nothing to work or plan, so a terminal on it draws a diagram, records
    /// the inventory or assesses an advisory.
    pub async fn open(&self, command: OpenTerminalCommand) -> Result<TerminalView> {
        if matches!(command.target, TerminalTarget::Project(_)) && !command.mode.works_on_a_project() {
            return Err(RekallError::illegal(
                "A terminal on a project only generates a diagram, records the inventory or assesses an advisory. Open one on a task to work or plan.",
            ));
        }
        self.host.open(opening_of(command)).await
    }

    pub async fn close(&self, id: Id) -> Result<()> {
        match self.host.close(id, CLOSED_FROM_THE_CONSOLE).await {
            Some(_) => Ok(()),
            None => Err(no_terminal(id)),
        }
    }
}

fn opening_of(command: OpenTerminalCommand) -> TerminalOpening {
    TerminalOpening {
        target: command.target,
        step_id: command.step_id,
        skip_permissions: command.skip_permissions,
        model: command.model,
        effort: command.effort,
        mode: command.mode,
    }
}

fn no_terminal(id: Id) -> RekallError {
    RekallError::not_found_msg(format!("No terminal with id {id}"))
}

#[cfg(test)]
#[path = "../../../tests/unit/terminal/service/terminal_service_tests.rs"]
mod tests;
