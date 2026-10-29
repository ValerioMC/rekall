use rekall_common::Id;
use rekall_console::catalog::TaskRefView;
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

impl From<TaskRefView> for TaskRef {
    fn from(view: TaskRefView) -> Self {
        Self {
            id: view.id,
            label: view.label,
            title: view.title,
            project_label: view.project_label,
            project_title: view.project_title,
            company_name: view.company_name,
            anchor: view.anchor,
        }
    }
}
