use rekall_kernel::Id;
use rekall_console::catalog::ProjectCommand;
use rekall_store::ProjectStatus;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectRequest {
    pub label: Option<String>,
    pub task_key: Option<String>,
    pub title: Option<String>,
    pub status: Option<ProjectStatus>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub blueprint_markdown: Option<String>,
    pub repo_folder: Option<String>,
    pub auto_commit: Option<bool>,
    pub main_branch: Option<String>,
    pub pull_request_flow: Option<bool>,
    pub company_id: Option<Id>,
}

impl From<ProjectRequest> for ProjectCommand {
    fn from(request: ProjectRequest) -> Self {
        Self {
            label: request.label,
            task_key: request.task_key,
            title: request.title,
            status: request.status,
            icon: request.icon,
            description: request.description,
            blueprint_markdown: request.blueprint_markdown,
            repo_folder: request.repo_folder,
            auto_commit: request.auto_commit,
            main_branch: request.main_branch,
            pull_request_flow: request.pull_request_flow,
            company_id: request.company_id,
        }
    }
}
