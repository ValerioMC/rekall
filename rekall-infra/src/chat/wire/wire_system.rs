use serde::Deserialize;

/// A `system` line, by its `subtype`; the retry fields are there only on `api_retry` and the
/// session fields only on `init`.
#[derive(Debug, Deserialize)]
pub(in crate::chat) struct WireSystem {
    pub(in crate::chat) subtype: String,
    pub(in crate::chat) attempt: Option<u64>,
    pub(in crate::chat) retry_delay_ms: Option<u64>,
    pub(in crate::chat) model: Option<String>,
    pub(in crate::chat) cwd: Option<String>,
    #[serde(rename = "permissionMode")]
    pub(in crate::chat) permission_mode: Option<String>,
    pub(in crate::chat) claude_code_version: Option<String>,
}
