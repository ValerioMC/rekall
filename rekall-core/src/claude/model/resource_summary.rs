use rekall_kernel::Instant;
use serde::Serialize;

use crate::claude::{ResourceKind, SkillSource};

/// What the console lists about a file without opening it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSummary {
    /// The identity: the file's path under the Claude folder.
    pub path: String,
    pub kind: ResourceKind,
    pub name: String,
    pub description: Option<String>,
    /// A memory's own `type` (`feedback`, `project`...), when its front matter names one.
    pub memory_type: Option<String>,
    pub bytes: u64,
    pub modified_at: Instant,
    pub editable: bool,
    /// Who provides a skill; none for every other kind of file.
    pub source: Option<SkillSource>,
}
