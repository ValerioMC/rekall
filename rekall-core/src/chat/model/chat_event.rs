use serde::Serialize;

use crate::chat::ChatBody;

/// One thing that happened in a chat session. `seq` counts from 1 within the session, so a client
/// that reconnects asks for what came after the last one it saw.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ChatEvent {
    pub seq: u64,
    #[serde(flatten)]
    pub body: ChatBody,
}
