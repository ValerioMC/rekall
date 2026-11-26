use std::collections::HashSet;

use rekall_core::commit::{ChangeFileKey, TaskChangesScope};
use rekall_kernel::Id;
use serde::Deserialize;

use crate::shared::{error::ApiResult, extract::Validator};

const MAX_BATCH_FILES: usize = 8;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskChangesFilesRequest {
    pub snapshot_id: Id,
    pub scope: TaskChangesScope,
    pub files: Vec<ChangeFileKey>,
}

impl TaskChangesFilesRequest {
    pub fn validate(&self) -> ApiResult<()> {
        Validator::new()
            .require(
                "files",
                (1..=MAX_BATCH_FILES).contains(&self.files.len()),
                format!("must contain between 1 and {MAX_BATCH_FILES} keys"),
            )
            .finish()?;
        let distinct = self.files.iter().collect::<HashSet<_>>().len() == self.files.len();
        Validator::new()
            .require("files", distinct, "must contain distinct repository and path keys")
            .require(
                "files",
                self.files.iter().all(|file| !file.path.is_empty()),
                "must contain non-empty paths",
            )
            .finish()
    }
}
