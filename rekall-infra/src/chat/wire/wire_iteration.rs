use serde::Deserialize;

/// What one API call of a turn consumed, as `usage.iterations` lists it.
#[derive(Debug, Default, Deserialize)]
pub(in crate::chat) struct WireIteration {
    #[serde(default)]
    pub(in crate::chat) input_tokens: u64,
    #[serde(default)]
    pub(in crate::chat) cache_creation_input_tokens: u64,
    #[serde(default)]
    pub(in crate::chat) cache_read_input_tokens: u64,
}
