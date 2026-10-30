use serde::Deserialize;

/// The raw model stream `--include-partial-messages` adds, reduced to what the window follows live.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(in crate::chat) enum WireStreamEvent {
    MessageStart { message: WireMessageStart },
    ContentBlockStart { content_block: WireBlockStart },
    ContentBlockDelta { delta: WireDelta },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub(in crate::chat) struct WireMessageStart {
    pub(in crate::chat) id: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(in crate::chat) enum WireBlockStart {
    Thinking,
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(in crate::chat) enum WireDelta {
    TextDelta { text: String },
    ThinkingDelta { thinking: String },
    #[serde(other)]
    Other,
}
