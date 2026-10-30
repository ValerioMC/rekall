use crate::chat::ChatEvent;

/// A sink for one chat session's events and the moment it ends. Implemented by the SSE handler.
pub trait ChatListener: Send + Sync {
    fn event(&self, event: &ChatEvent);
    fn ended(&self, exit_code: i32, detail: &str);
}
