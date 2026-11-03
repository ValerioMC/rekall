use std::time::Duration;

use super::*;

const GENEROUS: Duration = Duration::from_secs(20);

fn runner() -> ShellCheckRunner {
    ShellCheckRunner::new(LoginShellEnvironment::new(Some("/bin/sh"), Duration::from_secs(5)))
}

async fn run(command: &str, timeout: Duration) -> CheckRun {
    let folder = tempfile::tempdir().unwrap();
    runner().run(folder.path(), command, timeout).await.unwrap()
}

#[tokio::test]
async fn a_command_that_succeeds_exits_zero() {
    let run = run("true", GENEROUS).await;

    assert_eq!(run.outcome, CheckOutcome::Exited(0));
    assert!(run.outcome.passed());
}

#[tokio::test]
async fn a_command_that_fails_reports_its_exit_code() {
    let run = run("false", GENEROUS).await;

    assert_eq!(run.outcome, CheckOutcome::Exited(1));
    assert!(!run.outcome.passed());
}

#[tokio::test]
async fn standard_output_and_error_are_both_kept_with_the_exit_code() {
    let run = run("echo to-out; echo to-err 1>&2; exit 3", GENEROUS).await;

    assert_eq!(run.outcome, CheckOutcome::Exited(3));
    assert!(run.output_tail.contains("to-out") && run.output_tail.contains("to-err"), "{}", run.output_tail);
}

#[tokio::test]
async fn the_command_runs_in_the_given_folder() {
    let folder = tempfile::tempdir().unwrap();
    let canonical = folder.path().canonicalize().unwrap();

    let run = runner().run(folder.path(), "pwd", GENEROUS).await.unwrap();

    assert_eq!(run.output_tail, canonical.to_string_lossy());
}

#[tokio::test]
async fn a_command_past_its_timeout_is_ended_and_reported_as_timed_out() {
    let started = Instant::now();

    let run = run("echo started; sleep 30", Duration::from_millis(300)).await;

    assert_eq!(run.outcome, CheckOutcome::TimedOut);
    assert_eq!(run.outcome.exit_code(), None);
    assert!(run.output_tail.contains("started"));
    assert!(started.elapsed() < Duration::from_secs(10), "the run waited for the sleep");
}

#[tokio::test]
async fn a_timeout_ends_the_children_of_the_command_too() {
    let started = Instant::now();

    let run = run("sleep 30 & sleep 30", Duration::from_millis(300)).await;

    assert_eq!(run.outcome, CheckOutcome::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(10), "a child kept the output open");
}

#[tokio::test]
async fn a_background_process_left_behind_does_not_hold_the_run_open() {
    let started = Instant::now();

    let run = run("sleep 30 & echo done", GENEROUS).await;

    assert_eq!(run.outcome, CheckOutcome::Exited(0));
    assert!(started.elapsed() < Duration::from_secs(10), "the leftover process kept the output open");
}

#[tokio::test]
async fn a_folder_that_does_not_exist_is_refused() {
    let failure = runner().run(std::path::Path::new("/no/such/folder"), "true", GENEROUS).await.unwrap_err();

    assert!(failure.is_illegal_argument());
}
