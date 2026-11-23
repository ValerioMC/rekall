use crate::{task_decision, task_decision_detail, task_decision_line, task_decision_option};

/// A decision with everything that hangs from it, the children in the order they were written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionRecord {
    pub decision: task_decision::Model,
    pub options: Vec<task_decision_option::Model>,
    pub lines: Vec<task_decision_line::Model>,
    pub details: Vec<task_decision_detail::Model>,
}
