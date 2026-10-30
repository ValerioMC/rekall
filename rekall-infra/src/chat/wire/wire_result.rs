use serde::Deserialize;

/// The line that ends a turn.
#[derive(Debug, Deserialize)]
pub(in crate::chat) struct WireResult {
    #[serde(default)]
    pub(in crate::chat) is_error: bool,
    pub(in crate::chat) terminal_reason: Option<String>,
    pub(in crate::chat) duration_ms: Option<u64>,
    pub(in crate::chat) usage: Option<WireUsage>,
}

#[derive(Debug, Default, Deserialize)]
pub(in crate::chat) struct WireUsage {
    #[serde(default)]
    pub(in crate::chat) input_tokens: u64,
    #[serde(default)]
    pub(in crate::chat) cache_creation_input_tokens: u64,
    #[serde(default)]
    pub(in crate::chat) cache_read_input_tokens: u64,
    #[serde(default)]
    pub(in crate::chat) output_tokens: u64,
}
