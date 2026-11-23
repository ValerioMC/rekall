/// A new answer for an answered decision, a dated detail added to it, or both.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrectDecision {
    pub key: String,
    pub answer: Option<String>,
    pub detail: Option<String>,
}
