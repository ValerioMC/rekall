use rekall_common::{jstr, Id, RekallError, Result};

use crate::{in_read, load, Ctx};

use super::TerminalLaunch;

#[derive(Clone)]
pub struct TerminalLaunchService {
    ctx: Ctx,
}

impl TerminalLaunchService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// The folder is the project's `repo_folder`; a project without one is refused here.
    pub async fn resolve(&self, task_id: Id) -> Result<TerminalLaunch> {
        in_read!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            let project = load::project_of(tx.db(), &task).await?;
            let Some(folder) = project.repo_folder.as_deref().filter(|f| !jstr::is_blank(f)) else {
                return Err(RekallError::illegal(
                    "Set this project's folder on its page before opening a terminal here.",
                ));
            };
            Ok(TerminalLaunch {
                task_id,
                anchors: format!("project:{} task:{}", project.label, task.label),
                working_dir: jstr::strip(folder).to_string(),
                project_label: project.label.clone(),
                task_label: task.label.clone(),
                task_title: task.title.clone(),
            })
        })
    }
}
