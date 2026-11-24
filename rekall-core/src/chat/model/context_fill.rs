use serde::Serialize;

/// How much of its context window a session held at the end of a turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextFill {
    pub used_tokens: u64,
    pub window_tokens: u64,
}

impl ContextFill {
    /// The share held, rounded to a whole percent and capped at 100; 0 when the window is unknown.
    pub fn percent(&self) -> u8 {
        if self.window_tokens == 0 {
            return 0;
        }
        let rounded = (self.used_tokens as u128 * 100 + self.window_tokens as u128 / 2) / self.window_tokens as u128;
        rounded.min(100) as u8
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/chat/model/context_fill_tests.rs"]
mod tests;
