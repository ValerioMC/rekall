use super::spoken_line::SpokenLine;

/// The answer to an open or on-hold decision, with a detail and the lines it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnswerDecision {
    pub key: String,
    pub answer: Option<String>,
    pub detail: Option<String>,
    pub lines: Vec<SpokenLine>,
}
