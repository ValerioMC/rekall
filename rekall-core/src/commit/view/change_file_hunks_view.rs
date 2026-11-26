use serde::Serialize;

use super::{ChangeSourceId, DiffHunkView};
use crate::commit::model::change_file_key::ChangeFileKey;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeFileHunksView {
    pub file: ChangeFileKey,
    pub source: ChangeSourceId,
    pub hunks: Vec<DiffHunkView>,
}
