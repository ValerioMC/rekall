use rekall_console::decision::AddDecision;
use serde::Deserialize;

/// A decision the person writes by hand: the question and its answer, optionally under a topic and on one step.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AddDecisionRequest {
    pub topic: Option<String>,
    pub question: Option<String>,
    pub answer: Option<String>,
    pub step_number: Option<i32>,
}

impl AddDecisionRequest {
    pub fn into_command(self) -> AddDecision {
        AddDecision { topic: self.topic, question: self.question, answer: self.answer, step_number: self.step_number }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/decision/request/add_decision_request_tests.rs"]
mod tests;
