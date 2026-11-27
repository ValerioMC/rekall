use serde::{Deserialize, Serialize};

/// A model the installed Claude Code offers. `value` is what a session is started with. The effort
/// fields stay empty when the CLI leaves them out, which differs from its answering `false`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeModel {
    pub value: String,
    pub display_name: String,
    pub resolved_model: Option<String>,
    pub supports_effort: Option<bool>,
    pub supported_effort_levels: Option<Vec<String>>,
}
