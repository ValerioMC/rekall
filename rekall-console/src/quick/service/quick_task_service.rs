use rekall_core::claude::{TerminalMode, TerminalTarget};
use rekall_core::{in_write, Ctx, Tx};
use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::repository as repo;
use tracing::warn;

use crate::catalog::{TaskService, TaskView};
use crate::quick::rule::quick_task_rules::{label, task_request, title};
use crate::quick::{QuickTaskCommand, QuickTaskView};
use crate::terminal::{OpenTerminalCommand, TerminalService};

const LABEL_ATTEMPTS: usize = 20;

#[derive(Clone)]
pub struct QuickTaskService {
    ctx: Ctx,
    tasks: TaskService,
    terminals: TerminalService,
}

impl QuickTaskService {
    pub fn new(ctx: Ctx, tasks: TaskService, terminals: TerminalService) -> Self {
        Self { ctx, tasks, terminals }
    }

    /// A task with no steps whose description is the text, and a session on it. A session that
    /// cannot open takes the task away again, so a refused start leaves nothing behind.
    pub async fn create(&self, command: QuickTaskCommand) -> Result<QuickTaskView> {
        if jstr::is_blank(&command.text) {
            return Err(RekallError::illegal("Say what Claude should do before starting a quick task."));
        }
        let task = self.create_task(&command).await?;
        match self.terminals.open(opening_of(&command, task.id)).await {
            Ok(terminal) => Ok(QuickTaskView { task, terminal }),
            Err(refusal) => {
                self.discard(&task).await;
                Err(refusal)
            }
        }
    }

    async fn create_task(&self, command: &QuickTaskCommand) -> Result<TaskView> {
        in_write!(&self.ctx, |tx| {
            let free = free_label(&tx, command.project_id, &command.text).await?;
            self.tasks.create_in(&tx, task_request(command.project_id, &command.text, free)).await
        })
    }

    async fn discard(&self, task: &TaskView) {
        if let Err(failure) = self.tasks.delete(task.id).await {
            warn!(%failure, task = %task.label, "A quick task whose session did not open could not be removed");
        }
    }
}

fn opening_of(command: &QuickTaskCommand, task_id: Id) -> OpenTerminalCommand {
    OpenTerminalCommand {
        target: TerminalTarget::Task(task_id),
        step_id: None,
        skip_permissions: command.skip_permissions,
        model: command.model.clone(),
        effort: command.effort.clone(),
        mode: TerminalMode::Work,
        kind: command.kind,
    }
}

async fn free_label(tx: &Tx, project_id: Id, text: &str) -> Result<String> {
    let title = title(text);
    for attempt in 1..=LABEL_ATTEMPTS {
        let candidate = label(&title, attempt);
        if !repo::task::label_taken(tx.db(), project_id, &candidate, None).await? {
            return Ok(candidate);
        }
    }
    Err(RekallError::conflict(format!(
        "The project already has {LABEL_ATTEMPTS} tasks labelled like \"{title}\". Word the request differently."
    )))
}

#[cfg(test)]
#[path = "../../../tests/unit/quick/service/quick_task_service_tests.rs"]
mod tests;
