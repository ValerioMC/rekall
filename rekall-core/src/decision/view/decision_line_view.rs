use rekall_kernel::Instant;
use rekall_store::{task_decision_line, Speaker};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionLineView {
    pub speaker: Speaker,
    pub text: String,
    pub spoken_at: Instant,
}

impl DecisionLineView {
    pub fn of(line: &task_decision_line::Model) -> Self {
        Self { speaker: line.speaker, text: line.text.clone(), spoken_at: line.spoken_at }
    }
}
