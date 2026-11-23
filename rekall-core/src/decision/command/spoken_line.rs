use rekall_store::Speaker;

/// One line of the conversation a decision came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpokenLine {
    pub speaker: Speaker,
    pub text: String,
}
