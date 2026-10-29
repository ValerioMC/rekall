use serde::Serialize;

use super::ResourceSummary;

/// The memories of one place: the folder Claude Code was started in, or none at all for the
/// memories that apply everywhere.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryGroup {
    /// The project folder's name under `projects/`, or `global`.
    pub id: String,
    pub global: bool,
    pub working_directory: Option<String>,
    /// False when the working directory was guessed from the folder's name.
    pub resolved: bool,
    /// Whether the working directory is still on disk; a memory of one that is not is an orphan.
    pub folder_exists: bool,
    pub index: Option<ResourceSummary>,
    pub memories: Vec<ResourceSummary>,
}
