use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodexModel {
    pub model: String,
    pub display_name: String,
    pub supported_reasoning_efforts: Vec<CodexReasoningEffort>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodexReasoningEffort {
    pub reasoning_effort: String,
}
