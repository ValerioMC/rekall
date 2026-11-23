use rekall_kernel::{Id, Instant};
use rekall_store::{DecisionRecord, DecisionSource, DecisionState};

use super::decision_detail_view::DecisionDetailView;
use super::decision_line_view::DecisionLineView;
use super::decision_option_view::DecisionOptionView;

/// A decision as a context and the console hand it over. `answer` is set only once it is answered
/// and `hold_note` only while it is on hold: both live in one column of the store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionView {
    pub id: Id,
    pub key: String,
    pub topic: Option<String>,
    pub question: String,
    pub why: Option<String>,
    pub step_number: Option<i32>,
    pub step_title: Option<String>,
    pub state: DecisionState,
    pub answer: Option<String>,
    pub hold_note: Option<String>,
    pub previous_answer: Option<String>,
    pub changed_at: Option<Instant>,
    pub source: DecisionSource,
    pub options: Vec<DecisionOptionView>,
    pub lines: Vec<DecisionLineView>,
    pub details: Vec<DecisionDetailView>,
}

impl DecisionView {
    pub fn of(record: &DecisionRecord, step_title: Option<String>) -> Self {
        let decision = &record.decision;
        let (answer, hold_note) = if decision.state == DecisionState::OnHold { (None, decision.answer.clone()) } else { (decision.answer.clone(), None) };
        Self {
            id: decision.id,
            key: decision.key.clone(),
            topic: decision.topic.clone(),
            question: decision.question.clone(),
            why: decision.why.clone(),
            step_number: decision.step_number,
            step_title,
            state: decision.state,
            answer,
            hold_note,
            previous_answer: decision.previous_answer.clone(),
            changed_at: decision.changed_at,
            source: decision.source,
            options: record.options.iter().map(DecisionOptionView::of).collect(),
            lines: record.lines.iter().map(DecisionLineView::of).collect(),
            details: record.details.iter().map(DecisionDetailView::of).collect(),
        }
    }
}
