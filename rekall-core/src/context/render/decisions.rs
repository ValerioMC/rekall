//! What was settled about the task before it runs: the task's own decisions, then each step's.

use rekall_kernel::jstr;
use rekall_store::{DecisionSource, DecisionState};

use crate::decision::DecisionView;

use super::super::model::context_renderer::ContextRenderer;
use super::text::{attribute, truncate, MAX_DOCUMENT_CHARACTERS};

impl ContextRenderer {
    pub fn render_decisions(&self, decisions: &[DecisionView]) -> String {
        if decisions.is_empty() {
            return String::new();
        }
        let in_state = |state: DecisionState| decisions.iter().filter(|decision| decision.state == state).count();
        format!(
            "\n<decisions count=\"{}\" answered=\"{}\" open=\"{}\" on-hold=\"{}\">\n{}\n</decisions>\n",
            decisions.len(),
            in_state(DecisionState::Answered),
            in_state(DecisionState::Open),
            in_state(DecisionState::OnHold),
            truncate(Some(&scoped_lines(decisions)), MAX_DOCUMENT_CHARACTERS)
        )
    }
}

fn scoped_lines(decisions: &[DecisionView]) -> String {
    let mut lines: Vec<String> = decisions.iter().filter(|decision| decision.step_number.is_none()).map(record_line).collect();
    let mut step_numbers: Vec<i32> = decisions.iter().filter_map(|decision| decision.step_number).collect();
    step_numbers.sort_unstable();
    step_numbers.dedup();
    for number in step_numbers {
        let own: Vec<&DecisionView> = decisions.iter().filter(|decision| decision.step_number == Some(number)).collect();
        lines.push(step_tag(number, own[0].step_title.as_deref()));
        lines.extend(own.into_iter().map(record_line));
        lines.push("</step>".to_string());
    }
    lines.join("\n")
}

fn step_tag(number: i32, title: Option<&str>) -> String {
    let title = title.map(|title| format!(" title=\"{}\"", attribute(title))).unwrap_or_default();
    format!("<step n=\"{number}\"{title}>")
}

fn record_line(decision: &DecisionView) -> String {
    let edited = if decision.source == DecisionSource::Person { " [edited by you]" } else { "" };
    format!("{}: {}{edited}", decision.key, state_text(decision))
}

fn state_text(decision: &DecisionView) -> String {
    match decision.state {
        DecisionState::Answered => answered_text(decision),
        DecisionState::Open => format!("? {}", open_text(decision)),
        DecisionState::OnHold => format!("on hold: {}", jstr::collapse_whitespace(decision.hold_note.as_deref().unwrap_or_default())),
    }
}

fn answered_text(decision: &DecisionView) -> String {
    let mut text = jstr::collapse_whitespace(decision.answer.as_deref().unwrap_or_default());
    if let Some(previous) = decision.previous_answer.as_deref() {
        text.push_str(&format!(" (was {})", jstr::collapse_whitespace(previous)));
    }
    for detail in &decision.details {
        text.push_str(&format!(" + {}: {}", detail.added_at.datetime().format("%Y-%m-%d"), jstr::collapse_whitespace(&detail.text)));
    }
    text
}

fn open_text(decision: &DecisionView) -> String {
    if decision.options.is_empty() {
        return jstr::collapse_whitespace(&decision.question);
    }
    decision.options.iter().map(|option| jstr::collapse_whitespace(&option.label)).collect::<Vec<_>>().join(" | ")
}
