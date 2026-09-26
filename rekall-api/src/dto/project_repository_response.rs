use rekall_service::commit::RepositoryStatus;
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

impl ProjectRepositoryResponse {
    pub fn of(status: RepositoryStatus, auto_commit: bool) -> Self {
        Self {
            folder: status.folder,
            exists: status.exists,
            repository: status.repository,
            branch: status.branch,
            user_name: status.user_name,
            user_email: status.user_email,
            auto_commit,
        }
    }
}
