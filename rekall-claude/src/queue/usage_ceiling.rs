//! Decides whether the run queue may start something new, from a usage reading and the ceiling.
//!
//! The windows that count are the ones a session on the queue's model spends: the five-hour
//! session and the weekly total always, and the weekly Opus or Sonnet window when the queue names
//! that model. A window at or over the ceiling holds the queue until the latest reset among the
//! windows over it, plus a minute so the next reading is taken after Anthropic rolled it over. A
//! window over the ceiling with no reset time is looked at again after fifteen minutes. With a
//! ceiling set and no usable reading, the queue holds and looks again after five.

use chrono::TimeDelta;
use rekall_common::Instant;

use crate::usage::{ClaudeUsageView, Limit, UsageStatus};

pub const RESET_GRACE: TimeDelta = TimeDelta::minutes(1);
pub const NO_RESET_RECHECK: TimeDelta = TimeDelta::minutes(15);
pub const UNREADABLE_RECHECK: TimeDelta = TimeDelta::minutes(5);

const ALWAYS: [&str; 2] = ["session", "weekly_all"];

/// `clear` lets the queue go on; otherwise it holds until `resume_at`, for `reason`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub clear: bool,
    pub resume_at: Option<Instant>,
    pub reason: Option<String>,
}

impl Verdict {
    pub fn go() -> Self {
        Self { clear: true, resume_at: None, reason: None }
    }

    pub fn hold(resume_at: Instant, reason: String) -> Self {
        Self { clear: false, resume_at: Some(resume_at), reason: Some(reason) }
    }
}

pub fn check(usage: Option<&ClaudeUsageView>, ceiling_percent: Option<i32>, model: Option<&str>, now: Instant) -> Verdict {
    let Some(ceiling) = ceiling_percent else {
        return Verdict::go();
    };
    let Some(usage) = usage.filter(|u| u.status == UsageStatus::Ok && !u.limits.is_empty()) else {
        return Verdict::hold(
            now.plus(UNREADABLE_RECHECK),
            format!("Claude usage can't be read, so the {ceiling}% ceiling can't be checked."),
        );
    };
    let over: Vec<&Limit> = usage
        .limits
        .iter()
        .filter(|limit| counts(&limit.key, model))
        .filter(|limit| limit.percent >= f64::from(ceiling))
        .collect();
    if over.is_empty() {
        return Verdict::go();
    }
    // `Stream.max`: the first of equal maxima.
    let mut worst = over[0];
    for limit in &over[1..] {
        if limit.percent > worst.percent {
            worst = limit;
        }
    }
    let reason = format!(
        "{} is at {}%, at or over the {ceiling}% ceiling.",
        worst.label,
        java_round(worst.percent)
    );
    if over.iter().any(|limit| limit.resets_at.is_none()) {
        return Verdict::hold(now.plus(NO_RESET_RECHECK), reason);
    }
    let latest_reset = over.iter().filter_map(|limit| limit.resets_at).max().expect("every reset is known");
    let resume_at = latest_reset.plus(RESET_GRACE);
    Verdict::hold(if resume_at.is_after(&now) { resume_at } else { now.plus(RESET_GRACE) }, reason)
}

/// `Math.round(double)`: half up.
fn java_round(value: f64) -> i64 {
    (value + 0.5).floor() as i64
}

fn counts(window_key: &str, model: Option<&str>) -> bool {
    if ALWAYS.contains(&window_key) {
        return true;
    }
    model.is_some_and(|model| window_key == format!("weekly_{model}"))
}

#[cfg(test)]
mod tests {
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
}
