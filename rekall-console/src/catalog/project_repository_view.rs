use rekall_service::commit::RepositoryStatus;

/// What the project's folder is, git-wise, plus whether it auto-commits there.
#[derive(Clone, Debug)]
pub struct ProjectRepositoryView {
    pub folder: Option<String>,
    pub exists: bool,
    pub repository: bool,
    pub branch: Option<String>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub auto_commit: bool,
}

impl ProjectRepositoryView {
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
