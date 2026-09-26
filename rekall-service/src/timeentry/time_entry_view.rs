use rekall_common::{Id, Instant, Result};
use rekall_model::{project, task, time_entry};
use serde::Serialize;

use crate::load;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeEntryView {
    pub id: Id,
    pub task_id: Id,
    pub task_label: String,
    pub task_title: String,
    pub project_label: String,
    pub anchor: String,
    pub started_at: Instant,
    pub stopped_at: Option<Instant>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl TimeEntryView {
    pub fn of(entry: &time_entry::Model, task: &task::Model, project: &project::Model) -> Self {
        Self {
            id: entry.id,
            task_id: task.id,
            task_label: task.label.clone(),
            task_title: task.title.clone(),
            project_label: project.label.clone(),
            anchor: format!("project:{} task:{}", project.label, task.label),
            started_at: entry.started_at,
            stopped_at: entry.stopped_at,
            created_at: entry.created_at,
            updated_at: entry.updated_at,
        }
    }

    pub(super) async fn load(db: &impl sea_orm::ConnectionTrait, entry: &time_entry::Model) -> Result<Self> {
        let task = load::task_or_unknown(db, entry.task_id).await?;
        let project = load::project_of(db, &task).await?;
        Ok(Self::of(entry, &task, &project))
    }
}
