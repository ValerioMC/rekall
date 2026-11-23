/// A new answer, a dated note kept beside it, or both. An unanswered decision needs the answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditDecision {
    pub answer: Option<String>,
    pub note: Option<String>,
}
