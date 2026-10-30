use std::sync::Arc;

use rekall_core::chat::{ChatAttachment, ChatListener, ChatPipe};
use rekall_core::claude::{SessionKind, TerminalHost};
use rekall_kernel::{Id, RekallError, Result};

#[derive(Clone)]
pub struct ChatService {
    host: Arc<dyn TerminalHost>,
    pipe: Arc<dyn ChatPipe>,
}

impl ChatService {
    pub fn new(host: Arc<dyn TerminalHost>, pipe: Arc<dyn ChatPipe>) -> Self {
        Self { host, pipe }
    }

    pub fn attach(&self, id: Id, listener: Arc<dyn ChatListener>, after_seq: u64) -> Result<ChatAttachment> {
        self.require_chat(id)?;
        self.pipe.attach(id, listener, after_seq)
    }

    pub fn detach(&self, id: Id, token: u64) {
        self.pipe.detach(id, token)
    }

    pub fn send(&self, id: Id, text: &str) -> Result<()> {
        self.require_chat(id)?;
        self.pipe.send(id, text)
    }

    pub fn interrupt(&self, id: Id) -> Result<()> {
        self.require_chat(id)?;
        self.pipe.interrupt(id)
    }

    pub fn answer_permission(&self, id: Id, request_id: &str, allow: bool) -> Result<()> {
        self.require_chat(id)?;
        self.pipe.answer_permission(id, request_id, allow)
    }

    fn require_chat(&self, id: Id) -> Result<()> {
        let view = self.host.get(id).ok_or_else(|| RekallError::not_found_msg(format!("No chat session with id {id}")))?;
        match view.kind {
            SessionKind::Chat => Ok(()),
            SessionKind::Terminal => Err(RekallError::illegal("This is a terminal session, not a chat session.")),
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/chat/service/chat_service_tests.rs"]
mod tests;
