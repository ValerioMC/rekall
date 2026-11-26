//! Decides whether the run queue may start something new, from a usage reading and the ceiling.
//!
//! A window at or over the ceiling holds the queue until the latest reset, plus a minute.
//! Without a reset time it checks again after fifteen minutes, and without a reading after five.

use chrono::TimeDelta;
use rekall_core::claude::{ClaudeUsageView, SessionProvider, UsageStatus};
use rekall_core::codex::CodexUsage;
use rekall_kernel::Instant;

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

pub(super) fn check_claude(usage: &ClaudeUsageView, ceiling: i32, model: Option<&str>, now: Instant) -> Verdict {
    if usage.status != UsageStatus::Ok || usage.limits.is_empty() {
        return unreadable("Claude", ceiling, now);
    }
    let windows = usage
        .limits
        .iter()
        .filter(|limit| counts(&limit.key, model))
        .map(|limit| Window { label: &limit.label, percent: limit.percent, resets_at: limit.resets_at });
    check_windows(windows, ceiling, now)
}

/// A missing CLI goes through the launcher's failure path so the item records the setup reason.
pub(super) fn check_codex(usage: CodexUsage, ceiling: i32, now: Instant) -> Verdict {
    let (primary, secondary) = match usage {
        CodexUsage::Ok { primary, secondary } if primary.is_some() || secondary.is_some() => (primary, secondary),
        CodexUsage::CliMissing => return Verdict::go(),
        _ => return unreadable(SessionProvider::Codex.label(), ceiling, now),
    };
    let windows = [("Codex primary window", primary), ("Codex secondary window", secondary)]
        .into_iter()
        .filter_map(|(label, window)| window.map(|window| Window {
            label,
            percent: window.used_percent,
            resets_at: window.resets_at.and_then(Instant::from_epoch_seconds),
        }));
    check_windows(windows, ceiling, now)
}

fn unreadable(provider: &str, ceiling: i32, now: Instant) -> Verdict {
    Verdict::hold(
        now.plus(UNREADABLE_RECHECK),
        format!("{provider} usage can't be read, so the {ceiling}% ceiling can't be checked."),
    )
}

struct Window<'a> {
    label: &'a str,
    percent: f64,
    resets_at: Option<Instant>,
}

fn check_windows<'a>(windows: impl Iterator<Item = Window<'a>>, ceiling: i32, now: Instant) -> Verdict {
    let over: Vec<_> = windows.filter(|window| window.percent >= f64::from(ceiling)).collect();
    let Some(worst) = over.iter().rev().max_by(|left, right| left.percent.total_cmp(&right.percent)) else {
        return Verdict::go();
    };
    let reason = format!(
        "{} is at {}%, at or over the {ceiling}% ceiling.",
        worst.label,
        round_half_up(worst.percent)
    );
    if over.iter().any(|window| window.resets_at.is_none()) {
        return Verdict::hold(now.plus(NO_RESET_RECHECK), reason);
    }
    let latest_reset = over.iter().filter_map(|window| window.resets_at).max().expect("every reset is known");
    let resume_at = latest_reset.plus(RESET_GRACE);
    Verdict::hold(if resume_at.is_after(&now) { resume_at } else { now.plus(RESET_GRACE) }, reason)
}

/// Half up, so 79.5% reads as 80%.
fn round_half_up(value: f64) -> i64 {
    (value + 0.5).floor() as i64
}

fn counts(window_key: &str, model: Option<&str>) -> bool {
    if ALWAYS.contains(&window_key) {
        return true;
    }
    model.is_some_and(|model| window_key == format!("weekly_{model}"))
}

#[cfg(test)]
#[path = "../../tests/unit/queue/usage_ceiling_tests.rs"]
mod tests;
