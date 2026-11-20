use rekall_console::connection::TargetCheckView;
use serde::Serialize;

use super::gitlab_namespace_response::GitlabNamespaceResponse;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TargetCheckStatus {
    Found,
    NotFound,
    OutsideRoot,
    NoCredential,
    Unreachable,
}

/// What GitLab has at a typed path. Only the fields of the outcome are set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetCheckResponse {
    pub status: TargetCheckStatus,
    pub found: Option<GitlabNamespaceResponse>,
    pub nearest: Option<String>,
    pub root: Option<String>,
    pub reason: Option<String>,
}

impl TargetCheckResponse {
    fn of(status: TargetCheckStatus) -> Self {
        Self { status, found: None, nearest: None, root: None, reason: None }
    }
}

impl From<TargetCheckView> for TargetCheckResponse {
    fn from(view: TargetCheckView) -> Self {
        match view {
            TargetCheckView::Found(found) => Self { found: Some(found.into()), ..Self::of(TargetCheckStatus::Found) },
            TargetCheckView::NotFound { nearest } => Self { nearest, ..Self::of(TargetCheckStatus::NotFound) },
            TargetCheckView::OutsideRoot { root } => Self { root: Some(root), ..Self::of(TargetCheckStatus::OutsideRoot) },
            TargetCheckView::NoCredential => Self::of(TargetCheckStatus::NoCredential),
            TargetCheckView::Unreachable { reason } => Self { reason: Some(reason), ..Self::of(TargetCheckStatus::Unreachable) },
        }
    }
}
