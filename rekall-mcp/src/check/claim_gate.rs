use rekall_core::check::{CheckOutcome, CheckRan, CheckReport};

use crate::protocol::ToolError;
use crate::texts;

const NO_OUTPUT: &str = "(no output)";

/// Lets a claim through, with the line that says what passed, or refuses it. `left_as_it_was` says
/// what the refusal leaves untouched, so the session knows what to repeat after the fix.
pub fn admit(report: CheckReport, left_as_it_was: &str) -> Result<Option<String>, ToolError> {
    match report {
        CheckReport::NoChecks => Ok(None),
        CheckReport::Passed { runs } => Ok(Some(passed_line(&runs))),
        CheckReport::Failed { label, outcome, output_tail } => Err(ToolError::Failure(refusal(&label, outcome, &output_tail, left_as_it_was))),
    }
}

fn passed_line(runs: &[CheckRan]) -> String {
    let labels: Vec<&str> = runs.iter().map(|run| run.label.as_str()).collect();
    texts::CHECKS_PASSED_REPORT.replacen("%s", &labels.join(", "), 1)
}

fn refusal(label: &str, outcome: CheckOutcome, output_tail: &str, left_as_it_was: &str) -> String {
    let output = if output_tail.trim().is_empty() { NO_OUTPUT } else { output_tail };
    texts::CHECK_REFUSED_REPORT
        .replacen("%s", label, 1)
        .replacen("%s", &how_it_ended(outcome), 1)
        .replacen("%s", left_as_it_was, 1)
        .replacen("%s", output, 1)
}

fn how_it_ended(outcome: CheckOutcome) -> String {
    match outcome {
        CheckOutcome::Exited(code) => format!("failed with exit code {code}"),
        CheckOutcome::TimedOut => "timed out".to_string(),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/check/claim_gate_tests.rs"]
mod tests;
