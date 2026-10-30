use serde::Deserialize;

/// A `system` line, by its `subtype`; the retry fields are there only on `api_retry`.
#[derive(Debug, Deserialize)]
pub(in crate::chat) struct WireSystem {
    pub(in crate::chat) subtype: String,
    pub(in crate::chat) attempt: Option<u64>,
    pub(in crate::chat) retry_delay_ms: Option<u64>,
}
