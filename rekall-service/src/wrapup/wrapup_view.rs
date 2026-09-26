use rekall_common::{Id, Instant, Result};
use rekall_model::wrapup;
use rekall_model::{project, task, WrapupAuthor};
use serde::Serialize;

use crate::load;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WrapupView {
    pub id: Id,
    pub task_id: Id,
    pub task_label: String,
    pub task_title: String,
    pub project_label: String,
    pub anchor: String,
    pub body_markdown: String,
    pub written_by: WrapupAuthor,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl WrapupView {
    pub fn of(wrapup: &wrapup::Model, task: &task::Model, project: &project::Model) -> Self {
        Self {
            id: wrapup.id,
            task_id: task.id,
            task_label: task.label.clone(),
            task_title: task.title.clone(),
            project_label: project.label.clone(),
            anchor: format!("project:{} task:{}", project.label, task.label),
            body_markdown: wrapup.body_markdown.clone(),
            written_by: wrapup.written_by,
            created_at: wrapup.created_at,
            updated_at: wrapup.updated_at,
        }
    }

    pub async fn load(db: &impl sea_orm::ConnectionTrait, wrapup: &wrapup::Model) -> Result<Self> {
        let task = load::task_or_unknown(db, wrapup.task_id).await?;
        let project = load::project_of(db, &task).await?;
        Ok(Self::of(wrapup, &task, &project))
    }
}
