use rekall_kernel::Instant;
use serde::Deserialize;

/// A file's whole new text. `expected_modified_at` is the version the editor opened: null creates
/// a file that must not exist yet, a value rewrites one that has not changed since.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRequest {
    pub path: Option<String>,
    pub body: Option<String>,
    pub expected_modified_at: Option<Instant>,
}
