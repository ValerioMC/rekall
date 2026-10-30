use serde::Deserialize;

use super::{WireBlocks, WireControlRequest, WireResult, WireStreamEvent, WireSystem};

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
    },
    User {
        message: WireMessage,
        #[serde(default, rename = "isReplay")]
        is_replay: bool,
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
