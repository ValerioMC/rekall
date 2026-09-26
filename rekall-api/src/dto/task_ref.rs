use rekall_common::Id;
use rekall_model::{company, project, task};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRef {
    pub id: Id,
    pub label: String,
    pub title: String,
    pub project_label: String,
    pub project_title: String,
    pub company_name: String,
    pub anchor: String,
}

impl TaskRef {
    pub fn of(task: &task::Model, project: &project::Model, company: &company::Model) -> Self {
        Self {
            id: task.id,
            label: task.label.clone(),
            title: task.title.clone(),
            project_label: project.label.clone(),
            project_title: project.title.clone(),
            company_name: company.name.clone(),
            anchor: format!("project:{} task:{}", project.label, task.label),
        }
    }
}
