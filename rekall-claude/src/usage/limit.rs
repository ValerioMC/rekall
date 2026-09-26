use rekall_common::Instant;
use serde::Serialize;

use super::Severity;

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
