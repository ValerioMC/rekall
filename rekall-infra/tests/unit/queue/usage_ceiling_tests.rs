use super::*;
use crate::usage::Severity;

fn now() -> Instant {
    Instant::parse("2026-09-23T10:00:00Z").unwrap()
}

fn session_reset() -> Instant {
    now().plus_seconds(2 * 3600)
}

fn week_reset() -> Instant {
    now().plus_seconds(3 * 24 * 3600)
}

fn limit(key: &str, percent: f64, resets_at: Option<Instant>) -> Limit {
    Limit { key: key.into(), label: key.into(), percent, severity: Severity::Normal, resets_at }
}

fn reading(limits: Vec<Limit>) -> ClaudeUsageView {
    ClaudeUsageView::ok(limits, now())
}

#[test]
fn no_ceiling_never_holds() {
    assert!(check(Some(&ClaudeUsageView::unauthenticated()), None, None, now()).clear);
}

#[test]
fn under_the_ceiling_on_every_window_that_counts_goes_on() {
    let usage = reading(vec![limit("session", 40.0, Some(session_reset())), limit("weekly_all", 70.0, Some(week_reset()))]);
    assert!(check(Some(&usage), Some(80), None, now()).clear);
}

#[test]
fn the_session_window_at_the_ceiling_holds_until_it_resets_plus_the_grace() {
    let usage = reading(vec![limit("session", 80.0, Some(session_reset())), limit("weekly_all", 20.0, Some(week_reset()))]);
    let verdict = check(Some(&usage), Some(80), None, now());
    assert!(!verdict.clear);
    assert_eq!(verdict.resume_at, Some(session_reset().plus(RESET_GRACE)));
    let reason = verdict.reason.unwrap();
    assert!(reason.contains("session") && reason.contains("80%"));
}

#[test]
fn with_two_windows_over_the_queue_waits_for_the_later_reset() {
    let usage = reading(vec![limit("session", 95.0, Some(session_reset())), limit("weekly_all", 90.0, Some(week_reset()))]);
    assert_eq!(check(Some(&usage), Some(85), None, now()).resume_at, Some(week_reset().plus(RESET_GRACE)));
}

#[test]
fn a_models_weekly_window_counts_only_when_the_queue_runs_that_model() {
    let usage = reading(vec![limit("session", 10.0, Some(session_reset())), limit("weekly_opus", 99.0, Some(week_reset()))]);
    assert!(check(Some(&usage), Some(90), None, now()).clear);
    assert!(check(Some(&usage), Some(90), Some("sonnet"), now()).clear);
    assert!(!check(Some(&usage), Some(90), Some("opus"), now()).clear);
}

#[test]
fn an_unreadable_reading_holds_a_queue_that_has_a_ceiling() {
    let verdict = check(Some(&ClaudeUsageView::unavailable()), Some(80), None, now());
    assert!(!verdict.clear);
    assert_eq!(verdict.resume_at, Some(now().plus(UNREADABLE_RECHECK)));
}

#[test]
fn a_window_over_the_ceiling_with_no_reset_time_is_looked_at_again_later() {
    let usage = reading(vec![limit("session", 90.0, None)]);
    assert_eq!(check(Some(&usage), Some(80), None, now()).resume_at, Some(now().plus(NO_RESET_RECHECK)));
}

#[test]
fn a_reset_already_in_the_past_still_waits_out_the_grace() {
    let usage = reading(vec![limit("session", 90.0, Some(now().plus_seconds(-600)))]);
    assert_eq!(check(Some(&usage), Some(80), None, now()).resume_at, Some(now().plus(RESET_GRACE)));
}
