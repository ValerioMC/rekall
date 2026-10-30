use serde::Deserialize;
use serde_json::Value;

/// One content block of a message. A block of a type Rekall does not show is `Unknown`.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(in crate::chat) enum WireBlock {
    Text { text: String },
    Thinking {
        #[serde(default)]
        thinking: String,
    },
    ToolUse {
        id: String,
        name: String,
        #[serde(default)]
        input: Value,
    },
    ToolResult {
        tool_use_id: String,
        content: Option<WireToolOutput>,
        #[serde(default)]
        is_error: bool,
    },
    #[serde(other)]
    Unknown,
}

/// A tool's output: plain text, or parts of which only the text ones are read.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(in crate::chat) enum WireToolOutput {
    Text(String),
    Parts(Vec<WirePart>),
}

#[derive(Debug, Deserialize)]
pub(in crate::chat) struct WirePart {
    text: Option<String>,
}

impl WireToolOutput {
    pub(in crate::chat) fn text(&self) -> String {
        match self {
            WireToolOutput::Text(text) => text.clone(),
            WireToolOutput::Parts(parts) => parts.iter().filter_map(|part| part.text.as_deref()).collect::<Vec<_>>().join("\n"),
        }
    }
}
