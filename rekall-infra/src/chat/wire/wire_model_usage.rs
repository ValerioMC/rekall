use serde::Deserialize;

/// What one model did in a turn, from `modelUsage`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::chat) struct WireModelUsage {
    #[serde(default)]
    pub(in crate::chat) output_tokens: u64,
    pub(in crate::chat) context_window: Option<u64>,
}
