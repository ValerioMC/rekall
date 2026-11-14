use serde::Serialize;

use super::RepositoryChangesView;

/// Everything a task changed, repository by repository: the project's own folder first, then its
/// components. A repository the task did not touch is left out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskChangesView {
    pub repositories: Vec<RepositoryChangesView>,
}
