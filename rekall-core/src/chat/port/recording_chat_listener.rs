use std::sync::Mutex;

use crate::chat::{ChatEvent, ChatListener};

/// The `ChatListener` tests use: it keeps every event and the end it was told of.
#[derive(Default)]
pub struct RecordingChatListener {
    events: Mutex<Vec<ChatEvent>>,
    end: Mutex<Option<(i32, String)>>,
}

impl RecordingChatListener {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<ChatEvent> {
        self.events.lock().expect("never poisoned").clone()
    }

    pub fn end(&self) -> Option<(i32, String)> {
        self.end.lock().expect("never poisoned").clone()
    }
}

impl ChatListener for RecordingChatListener {
    fn event(&self, event: &ChatEvent) {
        self.events.lock().expect("never poisoned").push(event.clone());
    }

    fn ended(&self, exit_code: i32, detail: &str) {
        *self.end.lock().expect("never poisoned") = Some((exit_code, detail.to_string()));
    }
}
