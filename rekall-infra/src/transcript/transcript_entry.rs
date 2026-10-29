//! The part of a transcript line the ledger reads. Claude Code writes far more on each line; what
//! is not named here is skipped by the parser.

use rekall_kernel::Instant;
use rekall_core::token::TokenCount;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub(super) struct TranscriptEntry {
    #[serde(rename = "type")]
    pub kind: String,
    pub timestamp: Option<String>,
    #[serde(rename = "isSidechain", default)]
    pub is_sidechain: bool,
    pub message: Option<EntryMessage>,
}

#[derive(Debug, Deserialize)]
pub(super) struct EntryMessage {
    pub id: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub usage: Option<Usage>,
    #[serde(default, deserialize_with = "lenient")]
    pub content: Content,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum Content {
    Text(String),
    Blocks(Vec<Block>),
}

impl Default for Content {
    fn default() -> Self {
        Self::Blocks(Vec::new())
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub(super) enum Block {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        name: String,
        #[serde(default)]
        input: serde_json::Value,
    },
    #[serde(other)]
    Other,
}

/// The arguments the Rekall tools that move a session between tasks and steps take. Every other
/// tool's input has a shape of its own, so it is read only for a Rekall tool.
#[derive(Debug, Default, Deserialize)]
pub(super) struct ToolInput {
    pub anchors: Option<String>,
    pub step: Option<StepReference>,
    pub state: Option<String>,
}

/// A step as a tool call names it: its number, sent as a string or as a number, or its title.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum StepReference {
    Text(String),
    Number(u64),
}

impl StepReference {
    pub fn text(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Number(number) => number.to_string(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct Usage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

impl Usage {
    pub fn count(&self) -> TokenCount {
        TokenCount {
            input: self.input_tokens,
            output: self.output_tokens,
            cache_creation: self.cache_creation_input_tokens,
            cache_read: self.cache_read_input_tokens,
        }
    }
}

impl TranscriptEntry {
    /// The line, when it is one the ledger has a use for. Anything else, or a line cut short by a
    /// session still writing it, is nothing.
    pub fn parse(line: &str) -> Option<Self> {
        if !line.contains("\"usage\"") && !line.contains("/rk") {
            return None;
        }
        serde_json::from_str(line).ok()
    }

    pub fn at(&self) -> Option<Instant> {
        self.timestamp.as_deref().and_then(|text| Instant::parse(text).ok())
    }

    /// The text a person typed, joined across its blocks.
    pub fn prompt_text(&self) -> String {
        match self.message.as_ref().map(|message| &message.content) {
            Some(Content::Text(text)) => text.clone(),
            Some(Content::Blocks(blocks)) => blocks
                .iter()
                .filter_map(|block| match block {
                    Block::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
            None => String::new(),
        }
    }

    /// The Rekall tool calls on the line, by the tool's name without its server prefix.
    pub fn rekall_calls(&self) -> Vec<(&str, ToolInput)> {
        let blocks = match self.message.as_ref().map(|message| &message.content) {
            Some(Content::Blocks(blocks)) => blocks.as_slice(),
            _ => &[],
        };
        blocks
            .iter()
            .filter_map(|block| match block {
                Block::ToolUse { name, input } => {
                    let tool = name.rsplit("__").next().unwrap_or(name);
                    let input = tool.starts_with("rekall_").then(|| ToolInput::deserialize(input).ok()).flatten()?;
                    Some((tool, input))
                }
                _ => None,
            })
            .collect()
    }
}

/// A field Claude Code may one day write in another shape reads as empty rather than costing
/// the line, and the tokens on it, its parse.
fn lenient<'de, D: Deserializer<'de>, T: Deserialize<'de> + Default>(deserializer: D) -> Result<T, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(T::deserialize(value).unwrap_or_default())
}
