use serde::Deserialize;

use super::{WireBlocks, WireControlRequest, WireLocalCommand, WireResult, WireStreamEvent, WireSystem, WireToolUseResult};

/// One line of `claude`'s stdout, by its `type`. A type Rekall does not read is `Other`.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(in crate::chat) enum WireLine {
    System(WireSystem),
    StreamEvent {
        event: WireStreamEvent,
    },
    Assistant {
        message: WireMessage,
        local_command_source: Option<String>,
        local_command_run: Option<WireLocalCommand>,
    },
    User {
        message: WireMessage,
        #[serde(default, rename = "isReplay")]
        is_replay: bool,
        #[serde(default)]
        tool_use_result: Option<WireToolUseResult>,
    },
    Result(WireResult),
    ControlRequest {
        request_id: String,
        request: WireControlRequest,
    },
    ConversationReset {},
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub(in crate::chat) struct WireMessage {
    pub(in crate::chat) id: Option<String>,
    pub(in crate::chat) content: WireBlocks,
}
