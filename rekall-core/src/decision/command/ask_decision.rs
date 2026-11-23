use rekall_store::DecisionSource;

use super::offered_option::OfferedOption;
use super::spoken_line::SpokenLine;

/// A question to record on a task, or on one of its steps when `step_number` is given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskDecision {
    pub key: String,
    pub question: Option<String>,
    pub topic: Option<String>,
    pub why: Option<String>,
    pub step_number: Option<i32>,
    pub options: Vec<OfferedOption>,
    pub source: DecisionSource,
    pub lines: Vec<SpokenLine>,
}
