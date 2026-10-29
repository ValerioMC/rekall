use rekall_kernel::Instant;
use serde::Serialize;

use crate::claude::Severity;

/// One consumption window. `percent` is 0-100, already clamped; `resets_at` may be absent.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Limit {
    pub key: String,
    pub label: String,
    pub percent: f64,
    pub severity: Severity,
    pub resets_at: Option<Instant>,
}
