use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::TimeDelta;
use rekall_common::Instant;
use serde_json::Value;
use tokio::sync::{Mutex, Notify};
use tracing::{debug, info};

use super::{ClaudeUsageView, Limit, Severity, TokenSource, UsageReader, UsageStatus};

const CACHE_TTL: TimeDelta = TimeDelta::seconds(60);
const DEGRADED_TTL: TimeDelta = TimeDelta::seconds(10);
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
const RATE_LIMIT_HOLD: TimeDelta = TimeDelta::minutes(5);
const WARNING_AT: f64 = 80.0;
const CRITICAL_AT: f64 = 95.0;

const WINDOWS: [(&str, &str, &str); 4] = [
    ("five_hour", "session", "Session"),
    ("seven_day", "weekly_all", "Weekly · all models"),
    ("seven_day_opus", "weekly_opus", "Weekly · Opus"),
    ("seven_day_sonnet", "weekly_sonnet", "Weekly · Sonnet"),
];

struct State_ {
    cached: Option<ClaudeUsageView>,
    cached_at: Instant,
    last_good: Option<ClaudeUsageView>,
    hold_until: Instant,
}

pub struct ClaudeUsageService {
    credentials: Arc<dyn TokenSource>,
    http: reqwest::Client,
    usage_url: String,
    clock: rekall_service::Clock,
    state: Mutex<State_>,
    stopped: AtomicBool,
    shutdown: Notify,
}

impl ClaudeUsageService {
    pub fn new(credentials: Arc<dyn TokenSource>, usage_url: &str, clock: rekall_service::Clock) -> Self {
        Self {
            credentials,
            http: reqwest::Client::builder().connect_timeout(HTTP_TIMEOUT).build().expect("an HTTP client"),
            usage_url: usage_url.to_string(),
            clock,
            state: Mutex::new(State_ { cached: None, cached_at: Instant::EPOCH, last_good: None, hold_until: Instant::EPOCH }),
            stopped: AtomicBool::new(false),
            shutdown: Notify::new(),
        }
    }

    fn now(&self) -> Instant {
        (self.clock)()
    }

    fn on_hold(&self, state: &State_) -> bool {
        self.now().is_before(&state.hold_until)
    }

    fn held(state: &State_) -> ClaudeUsageView {
        match &state.last_good {
            Some(good) => good.held_until(state.hold_until),
            None => ClaudeUsageView::rate_limited(state.hold_until),
        }
    }

    fn degraded(state: &State_) -> ClaudeUsageView {
        state.last_good.clone().unwrap_or_else(ClaudeUsageView::unavailable)
    }

    async fn read_and_cache(&self, state: &mut State_) -> ClaudeUsageView {
        let fresh = self.fetch(state).await;
        state.cached = Some(fresh.clone());
        state.cached_at = self.now();
        if fresh.status == UsageStatus::Ok {
            state.last_good = Some(fresh.clone());
        }
        fresh
    }

    async fn fetch(&self, state: &mut State_) -> ClaudeUsageView {
        let Some(token) = self.credentials.access_token() else {
            return ClaudeUsageView::unauthenticated();
        };
        let request = self
            .http
            .get(&self.usage_url)
            .timeout(HTTP_TIMEOUT)
            .header("Authorization", format!("Bearer {token}"))
            .header("anthropic-beta", "oauth-2025-04-20")
            .header("Accept", "application/json")
            .send();
        let response = tokio::select! {
            response = request => response,
            _ = self.shutdown.notified() => {
                debug!("Claude usage read cut off by shutdown");
                return Self::degraded(state);
            }
        };
        let response = match response {
            Ok(response) => response,
            Err(failure) => {
                debug!("Claude usage endpoint unreachable: {failure}");
                return Self::degraded(state);
            }
        };
        let status = response.status().as_u16();
        if status == 401 || status == 403 {
            return ClaudeUsageView::unauthenticated();
        }
        if status == 429 {
            state.hold_until = self.now().plus(retry_after(response.headers().get("Retry-After")));
            info!("Anthropic rate limited the usage read; holding until {}", state.hold_until);
            return Self::held(state);
        }
        if status != 200 {
            debug!("Claude usage endpoint returned {status}");
            return Self::degraded(state);
        }
        match response.text().await.ok().and_then(|body| parse(&body)) {
            Some(limits) => ClaudeUsageView::ok(limits, Instant::now()),
            None => Self::degraded(state),
        }
    }
}

#[async_trait]
impl UsageReader for ClaudeUsageService {
    async fn current(&self) -> ClaudeUsageView {
        let mut state = self.state.lock().await;
        if self.stopped.load(Ordering::SeqCst) {
            return Self::degraded(&state);
        }
        if self.on_hold(&state) {
            return Self::held(&state);
        }
        if let Some(cached) = &state.cached {
            let ttl = if cached.status == UsageStatus::Ok { CACHE_TTL } else { DEGRADED_TTL };
            if state.cached_at.until(&self.now()) < ttl {
                return cached.clone();
            }
        }
        self.read_and_cache(&mut state).await
    }

    /// A reading taken now, whatever is cached. A rate-limit hold still stands.
    async fn refresh(&self) -> ClaudeUsageView {
        let mut state = self.state.lock().await;
        if self.stopped.load(Ordering::SeqCst) {
            return Self::degraded(&state);
        }
        if self.on_hold(&state) {
            return Self::held(&state);
        }
        self.read_and_cache(&mut state).await
    }

    /// Cut an in-flight poll on shutdown so graceful shutdown has no blocking request to wait on.
    fn release_on_shutdown(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        self.shutdown.notify_waiters();
    }
}

/// `Retry-After` as Anthropic sends it, in seconds; anything else means the default hold.
fn retry_after(header: Option<&reqwest::header::HeaderValue>) -> TimeDelta {
    match header.and_then(|h| h.to_str().ok()).and_then(|v| v.trim().parse::<i64>().ok()) {
        Some(seconds) if seconds > 0 => TimeDelta::seconds(seconds),
        _ => RATE_LIMIT_HOLD,
    }
}

fn parse(body: &str) -> Option<Vec<Limit>> {
    let root: Value = serde_json::from_str(body).ok()?;
    let mut limits = Vec::new();
    for (field, key, label) in WINDOWS {
        let Some(node) = root.get(field).filter(|n| !n.is_null()) else { continue };
        let Some(utilization) = node.get("utilization").filter(|u| !u.is_null()) else { continue };
        let percent = utilization.as_f64().unwrap_or(0.0).clamp(0.0, 100.0);
        limits.push(Limit {
            key: key.into(),
            label: label.into(),
            percent,
            severity: severity_of(percent),
            resets_at: node.get("resets_at").and_then(Value::as_str).and_then(|t| Instant::parse(t).ok()),
        });
    }
    Some(limits)
}

fn severity_of(percent: f64) -> Severity {
    if percent >= CRITICAL_AT {
        Severity::Critical
    } else if percent >= WARNING_AT {
        Severity::Warning
    } else {
        Severity::Normal
    }
}

#[cfg(test)]
#[path = "claude_usage_service_tests.rs"]
mod tests;
