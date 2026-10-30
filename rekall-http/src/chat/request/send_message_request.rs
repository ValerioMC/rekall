use serde::Deserialize;

use crate::shared::extract::Validator;
use crate::shared::ApiResult;

pub const MESSAGE_MAX_CHARACTERS: usize = 20_000;

/// What the console sends to talk to a chat session.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SendMessageRequest {
    pub text: Option<String>,
}

impl SendMessageRequest {
    /// The message without the blanks around it; one that is empty or longer than
    /// [`MESSAGE_MAX_CHARACTERS`] is refused.
    pub fn trimmed_text(&self) -> ApiResult<String> {
        Validator::new()
            .not_blank("text", self.text.as_deref())
            .at_most("text", self.text.as_deref().map(str::trim), MESSAGE_MAX_CHARACTERS)
            .finish()?;
        Ok(self.text.as_deref().unwrap_or_default().trim().to_string())
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/chat/request/send_message_request_tests.rs"]
mod tests;
