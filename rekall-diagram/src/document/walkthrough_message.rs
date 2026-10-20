use serde::{Deserialize, Serialize};

/// What one message of a walkthrough tells the reader: what happens and why, what goes in, what
/// comes back, and the sheet (by its index in `sheets`) that walks through the call in detail.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WalkthroughMessage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opens: Option<usize>,
}

impl WalkthroughMessage {
    pub fn is_empty(&self) -> bool {
        self.explain.is_none() && self.input.is_none() && self.output.is_none() && self.opens.is_none()
    }
}
