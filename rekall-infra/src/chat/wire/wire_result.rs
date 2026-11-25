use std::collections::BTreeMap;

use serde::Deserialize;

use super::{WireIteration, WireModelUsage};

/// The line that ends a turn.
#[derive(Debug, Deserialize)]
pub(in crate::chat) struct WireResult {
    #[serde(default)]
    pub(in crate::chat) is_error: bool,
    pub(in crate::chat) terminal_reason: Option<String>,
    pub(in crate::chat) duration_ms: Option<u64>,
    pub(in crate::chat) usage: Option<WireUsage>,
    pub(in crate::chat) total_cost_usd: Option<f64>,
    #[serde(default, rename = "modelUsage")]
    pub(in crate::chat) model_usage: BTreeMap<String, WireModelUsage>,
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
    #[serde(default, deserialize_with = "tolerate_null_iterations")]
    pub(in crate::chat) iterations: Vec<WireIteration>,
}

fn tolerate_null_iterations<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<WireIteration>, D::Error> {
    Ok(Option::<Vec<WireIteration>>::deserialize(deserializer)?.unwrap_or_default())
}
