use serde::Deserialize;
use serde_json::Value;

/// What `claude` asks its host. Only the permission prompt is read.
#[derive(Debug, Deserialize)]
#[serde(tag = "subtype", rename_all = "snake_case")]
pub(in crate::chat) enum WireControlRequest {
    CanUseTool {
        tool_name: String,
        #[serde(default)]
        input: Value,
        tool_use_id: Option<String>,
    },
    #[serde(other)]
    Other,
}
