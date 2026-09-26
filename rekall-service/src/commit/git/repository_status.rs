/// What a project's folder is, git-wise. `branch` is absent on a detached head; the identity is
/// absent when git has none configured there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryStatus {
    pub folder: Option<String>,
    pub exists: bool,
    pub repository: bool,
    pub branch: Option<String>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
}

impl RepositoryStatus {
    pub fn unset() -> Self {
        Self { folder: None, exists: false, repository: false, branch: None, user_name: None, user_email: None }
    }

    pub fn missing(folder: &str) -> Self {
        Self { folder: Some(folder.to_string()), ..Self::unset() }
    }

    pub fn not_a_repository(folder: &str) -> Self {
        Self { folder: Some(folder.to_string()), exists: true, ..Self::unset() }
    }

    /// Whether git could commit here as someone: a repository with an email to sign with.
    pub fn can_commit(&self) -> bool {
        self.repository && self.user_email.is_some()
    }
}
