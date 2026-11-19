//! The chat view of a `claude` session: what happens in one, as events a client can follow, and
//! the ports a client reaches the session through. The process behind them is `rekall-infra`'s.

mod model;
mod port;

pub use model::chat_attachment::ChatAttachment;
pub use model::chat_body::{ChatBody, NoticeKind, TurnOutcome, TurnUsage, TOOL_EXCERPT_MAX_CHARACTERS};
pub use model::chat_event::ChatEvent;
pub use model::tool_input::ToolInput;
pub use port::chat_listener::ChatListener;
pub use port::chat_pipe::ChatPipe;
#[cfg(any(test, feature = "testing"))]
pub use port::in_memory_chat_pipe::{ChatCall, InMemoryChatPipe};
#[cfg(any(test, feature = "testing"))]
pub use port::recording_chat_listener::RecordingChatListener;
