use std::time::Duration;

use super::*;

const LEFT_AS_IT_WAS: &str = "The step stays as it was.";

fn ran(label: &str) -> CheckRan {
    CheckRan { label: label.into(), duration: Duration::from_millis(5) }
}

fn failed(outcome: CheckOutcome, output_tail: &str) -> CheckReport {
    CheckReport::Failed { label: "ui lint".into(), outcome, output_tail: output_tail.into() }
}

fn refusal_of(report: CheckReport) -> String {
    match admit(report, LEFT_AS_IT_WAS) {
        Err(ToolError::Failure(text)) => text,
        other => panic!("expected a refusal, got {other:?}"),
    }
}

#[test]
fn no_checks_lets_the_claim_through_without_a_line() {
    assert_eq!(admit(CheckReport::NoChecks, LEFT_AS_IT_WAS).unwrap(), None);
}

#[test]
fn passed_checks_let_the_claim_through_and_are_named() {
    let line = admit(CheckReport::Passed { runs: vec![ran("backend tests"), ran("ui lint")] }, LEFT_AS_IT_WAS).unwrap();

    assert_eq!(line.as_deref(), Some("Checks passed: backend tests, ui lint."));
}

#[test]
fn a_failed_check_refuses_with_its_name_exit_code_output_and_what_stays() {
    let text = refusal_of(failed(CheckOutcome::Exited(2), "lint: 3 errors"));

    assert!(text.contains("\"ui lint\" failed with exit code 2"), "{text}");
    assert!(text.contains("lint: 3 errors"), "{text}");
    assert!(text.contains(LEFT_AS_IT_WAS), "{text}");
    assert!(text.contains("Do not change the check"), "{text}");
}

#[test]
fn a_check_that_timed_out_says_so() {
    assert!(refusal_of(failed(CheckOutcome::TimedOut, "still compiling")).contains("\"ui lint\" timed out"));
}

#[test]
fn a_check_that_printed_nothing_says_there_was_no_output() {
    assert!(refusal_of(failed(CheckOutcome::Exited(1), "  \n")).contains("(no output)"));
}
