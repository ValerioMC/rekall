use rekall_console::decision::EditDecision;
use serde::Deserialize;

/// A new answer, a note kept beside it, or both.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EditDecisionRequest {
    pub answer: Option<String>,
    pub note: Option<String>,
}

impl EditDecisionRequest {
    pub fn into_command(self) -> EditDecision {
        EditDecision { answer: self.answer, note: self.note }
    }
}
