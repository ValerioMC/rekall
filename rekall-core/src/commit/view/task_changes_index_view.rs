use rekall_kernel::Id;
use serde::Serialize;

use super::{ChangedFileMetadata, RepositoryChangesView};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskChangesIndexView {
    pub snapshot_id: Id,
    pub repositories: Vec<RepositoryChangesView<Vec<ChangedFileMetadata>>>,
}
