use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use rekall_kernel::{Id, RekallError, Result};

use crate::chat::{ChatAttachment, ChatBody, ChatEvent, ChatListener, ChatPipe};
use crate::claude::TerminalView;

/// What a client did to a session of the `InMemoryChatPipe`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatCall {
    Sent { id: Id, text: String },
    Interrupted { id: Id },
    Answered { id: Id, request_id: String, allow: bool },
}

struct Session {
    view: TerminalView,
    events: Vec<ChatEvent>,
    listeners: Vec<(u64, Arc<dyn ChatListener>)>,
}

/// The `ChatPipe` tests use: sessions held in memory, fed by [`InMemoryChatPipe::emit`], that keep a
/// record of what clients did to them.
#[derive(Default)]
pub struct InMemoryChatPipe {
    sessions: Mutex<HashMap<Id, Session>>,
    calls: Mutex<Vec<ChatCall>>,
    next_token: Mutex<u64>,
}

impl InMemoryChatPipe {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_session(&self, view: TerminalView) {
        self.sessions().insert(view.id, Session { view, events: Vec::new(), listeners: Vec::new() });
    }

    /// Records `body` as the session's next event and hands it to whoever is attached.
    pub fn emit(&self, id: Id, body: ChatBody) -> Result<ChatEvent> {
        let mut sessions = self.sessions();
        let session = sessions.get_mut(&id).ok_or_else(|| no_session(id))?;
        let event = ChatEvent { seq: session.events.len() as u64 + 1, body };
        session.events.push(event.clone());
        for (_, listener) in &session.listeners {
            listener.event(&event);
        }
        Ok(event)
    }

    pub fn calls(&self) -> Vec<ChatCall> {
        self.calls.lock().expect("never poisoned").clone()
    }

    fn sessions(&self) -> MutexGuard<'_, HashMap<Id, Session>> {
        self.sessions.lock().expect("never poisoned")
    }

    fn record(&self, id: Id, call: ChatCall) -> Result<()> {
        if !self.sessions().contains_key(&id) {
            return Err(no_session(id));
        }
        self.calls.lock().expect("never poisoned").push(call);
        Ok(())
    }
}

impl ChatPipe for InMemoryChatPipe {
    fn attach(&self, id: Id, listener: Arc<dyn ChatListener>, after_seq: u64) -> Result<ChatAttachment> {
        let mut sessions = self.sessions();
        let session = sessions.get_mut(&id).ok_or_else(|| no_session(id))?;
        let mut next = self.next_token.lock().expect("never poisoned");
        *next += 1;
        session.listeners.push((*next, listener));
        let backlog = session.events.iter().filter(|event| event.seq > after_seq).cloned().collect();
        Ok(ChatAttachment { view: session.view.clone(), backlog, token: *next })
    }

    fn detach(&self, id: Id, token: u64) {
        if let Some(session) = self.sessions().get_mut(&id) {
            session.listeners.retain(|(held, _)| *held != token);
        }
    }

    fn send(&self, id: Id, text: &str) -> Result<()> {
        self.record(id, ChatCall::Sent { id, text: text.to_string() })
    }

    fn interrupt(&self, id: Id) -> Result<()> {
        self.record(id, ChatCall::Interrupted { id })
    }

    fn answer_permission(&self, id: Id, request_id: &str, allow: bool) -> Result<()> {
        self.record(id, ChatCall::Answered { id, request_id: request_id.to_string(), allow })
    }
}

fn no_session(id: Id) -> RekallError {
    RekallError::conflict(format!("The chat session {id} has closed."))
}

#[cfg(test)]
#[path = "../../../tests/unit/chat/port/in_memory_chat_pipe_tests.rs"]
mod tests;
