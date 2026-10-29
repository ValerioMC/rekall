use rekall_common::{jstr, RekallError, Result};
use rekall_store::{project, task};
use rekall_store::repository as repo;

use crate::{in_read, load, Ctx};

use super::{TerminalConnection, TerminalLaunch, TerminalTarget};

#[derive(Clone)]
pub struct TerminalLaunchService {
    ctx: Ctx,
}

impl TerminalLaunchService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// The folder is the project's `repo_folder`; a project without one is refused here.
    pub async fn resolve(&self, target: TerminalTarget) -> Result<TerminalLaunch> {
        in_read!(&self.ctx, |tx| {
            let (project, task) = match target {
                TerminalTarget::Task(task_id) => {
                    let task = load::task_or_unknown(tx.db(), task_id).await?;
                    task.require_workable("is never run")?;
                    (load::project_of(tx.db(), &task).await?, Some(task))
                }
                TerminalTarget::Project(project_id) => (repo::project::require(tx.db(), project_id).await?, None),
            };
            let bindings = repo::project_connection::list_of_project(tx.db(), project.id).await?;
            let connections = bindings.iter().map(|(binding, connection)| TerminalConnection::of(binding, connection)).collect();
            launch_of(&project, task.as_ref(), connections)
        })
    }
}

fn launch_of(project: &project::Model, task: Option<&task::Model>, connections: Vec<TerminalConnection>) -> Result<TerminalLaunch> {
    let Some(folder) = project.repo_folder.as_deref().filter(|f| !jstr::is_blank(f)) else {
        return Err(RekallError::illegal("Set this project's folder on its page before opening a terminal here."));
    };
    Ok(TerminalLaunch {
        project_id: project.id,
        task_id: task.map(|task| task.id),
        anchors: task.map_or_else(|| project.anchor(), |task| task.anchor(&project.label)),
        working_dir: jstr::strip(folder).to_string(),
        project_label: project.label.clone(),
        task_label: task.map(|task| task.label.clone()),
        task_title: task.map(|task| task.title.clone()),
        connections,
    })
}
