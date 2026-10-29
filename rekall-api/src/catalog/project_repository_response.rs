use rekall_console::catalog::ProjectRepositoryView;
use serde::Serialize;

/// What the project's folder is, git-wise, plus whether it auto-commits there.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRepositoryResponse {
    pub folder: Option<String>,
    pub exists: bool,
    pub repository: bool,
    pub branch: Option<String>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub auto_commit: bool,
}

impl From<ProjectRepositoryView> for ProjectRepositoryResponse {
    fn from(view: ProjectRepositoryView) -> Self {
        Self {
            folder: view.folder,
            exists: view.exists,
            repository: view.repository,
            branch: view.branch,
            user_name: view.user_name,
            user_email: view.user_email,
            auto_commit: view.auto_commit,
        }
    }
}
