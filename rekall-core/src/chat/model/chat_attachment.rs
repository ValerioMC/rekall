use crate::chat::ChatEvent;
use crate::claude::TerminalView;

/// What a client gets on attaching to a chat session: the session, the events it missed, and the
/// token that detaches it.
#[derive(Clone, Debug)]
pub struct ChatAttachment {
    pub view: TerminalView,
    pub backlog: Vec<ChatEvent>,
    pub token: u64,
}
