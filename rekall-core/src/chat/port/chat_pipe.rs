use std::sync::Arc;

use rekall_kernel::{Id, Result};

use crate::chat::{ChatAttachment, ChatListener};

/// One live chat session: a client attaches to follow its events and sends it messages, a stop and
/// the answers to its permission prompts.
pub trait ChatPipe: Send + Sync {
    /// Gives `listener` every event after `after_seq` that the session still remembers, then
    /// follows it live; the token detaches the listener.
    fn attach(&self, id: Id, listener: Arc<dyn ChatListener>, after_seq: u64) -> Result<ChatAttachment>;

    fn detach(&self, id: Id, token: u64);

    fn send(&self, id: Id, text: &str) -> Result<()>;

    /// Ends the turn in progress and leaves the session usable.
    fn interrupt(&self, id: Id) -> Result<()>;

    fn answer_permission(&self, id: Id, request_id: &str, allow: bool) -> Result<()>;
}
