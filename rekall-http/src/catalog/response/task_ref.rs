use rekall_kernel::Id;
use rekall_console::catalog::TaskRefView;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRef {
    pub id: Id,
    pub code: String,
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
            code: view.code,
            title: view.title,
            project_label: view.project_label,
            project_title: view.project_title,
            company_name: view.company_name,
            anchor: view.anchor,
        }
    }
}
