//! The logged-in account's Claude usage, fetched from Anthropic's OAuth usage endpoint and shaped
//! for the console meter. The four drawn windows are kept in a fixed order, each with a severity
//! from its percentage. A good read is cached for sixty seconds, a degraded one for ten, so a token
//! that was not readable at launch is retried on the next poll. A failed fetch falls back to the
//! last good one so a blip does not blank the meter, and `refresh` skips the cache when the person
//! asks for a new reading.
//!
//! A 429 is the one answer that is obeyed rather than retried: Anthropic's edge rate limits this
//! client, and every further request while the block stands extends it. The `Retry-After` it
//! carries (five minutes when it carries none) becomes a hold during which neither a poll nor a
//! refresh goes out, and the console is told when it ends.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::TimeDelta;
use rekall_api::ApiError;
use rekall_common::{Instant, RekallError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{Mutex, Notify};
use tracing::{debug, info};

use crate::credentials::ClaudeCredentials;
use crate::ClaudeState;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UsageStatus {
    Ok,
    Unauthenticated,
    RateLimited,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Normal,
    Warning,
    Critical,
}

/// One consumption window. `percent` is 0-100, already clamped; `resets_at` may be absent.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Limit {
    pub key: String,
    pub label: String,
    pub percent: f64,
    pub severity: Severity,
    pub resets_at: Option<Instant>,
}

/// What the console needs to draw the meter. `retry_at` is set whenever a wait is in force,
/// whatever the status: a last good reading served during one carries it too.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeUsageView {
    pub status: UsageStatus,
    pub limits: Vec<Limit>,
    pub fetched_at: Instant,
    pub retry_at: Option<Instant>,
}

impl ClaudeUsageView {
    pub fn ok(limits: Vec<Limit>, fetched_at: Instant) -> Self {
        Self { status: UsageStatus::Ok, limits, fetched_at, retry_at: None }
    }

    pub fn unauthenticated() -> Self {
        Self { status: UsageStatus::Unauthenticated, limits: Vec::new(), fetched_at: Instant::now(), retry_at: None }
    }

    pub fn rate_limited(retry_at: Instant) -> Self {
        Self { status: UsageStatus::RateLimited, limits: Vec::new(), fetched_at: Instant::now(), retry_at: Some(retry_at) }
    }

    pub fn unavailable() -> Self {
        Self { status: UsageStatus::Unavailable, limits: Vec::new(), fetched_at: Instant::now(), retry_at: None }
    }

    /// The same reading, marked as one that cannot be refreshed before `retry_at`.
    pub fn held_until(&self, retry_at: Instant) -> Self {
        Self { retry_at: Some(retry_at), ..self.clone() }
    }
}

/// What the meter and the run queue read usage through.
#[async_trait]
pub trait UsageReader: Send + Sync {
    async fn current(&self) -> ClaudeUsageView;
    async fn refresh(&self) -> ClaudeUsageView;
    fn release_on_shutdown(&self) {}
}

/// Where the token comes from: the real credentials, or what a test says.
pub trait TokenSource: Send + Sync {
    fn access_token(&self) -> Option<String>;
}

impl TokenSource for ClaudeCredentials {
    fn access_token(&self) -> Option<String> {
        ClaudeCredentials::access_token(self)
    }
}

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

// ---------------------------------------------------------------------------------- HTTP

pub fn routes() -> Router<ClaudeState> {
    Router::new().route("/api/claude/usage", get(usage))
}

#[derive(Deserialize)]
struct UsageQuery {
    refresh: Option<String>,
}

/// Always 200; whether the figures are real is carried in `status`. `?refresh=true` is the
/// meter's "check again".
async fn usage(State(state): State<ClaudeState>, Query(query): Query<UsageQuery>) -> Result<Json<ClaudeUsageView>, ApiError> {
    let refresh = match query.refresh.as_deref() {
        None => false,
        Some(value) => parse_boolean(value).ok_or_else(|| {
            ApiError::Domain(RekallError::internal(
                "MethodArgumentTypeMismatchException",
                format!(
                    "Method parameter 'refresh': Failed to convert value of type 'java.lang.String' to required type \
                     'boolean'; Invalid boolean value [{value}]"
                ),
            ))
        })?,
    };
    Ok(Json(if refresh { state.usage.refresh().await } else { state.usage.current().await }))
}

/// Spring's `StringToBooleanConverter`.
fn parse_boolean(value: &str) -> Option<bool> {
    match value.trim().to_lowercase().as_str() {
        "" => Some(false),
        "true" | "on" | "yes" | "1" => Some(true),
        "false" | "off" | "no" | "0" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex as StdMutex;

    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use super::*;

    const SAMPLE: &str = r#"{"five_hour":{"utilization":80.0,"resets_at":"2026-09-08T22:20:00.100547+00:00"},
         "seven_day":{"utilization":50.0,"resets_at":"2026-09-10T14:00:00.100568+00:00"},
         "seven_day_opus":{"utilization":97.5,"resets_at":"2026-09-10T14:00:00+00:00"},
         "seven_day_sonnet":null}"#;

    struct Token(StdMutex<Vec<Option<String>>>);

    impl TokenSource for Token {
        fn access_token(&self) -> Option<String> {
            let mut queue = self.0.lock().unwrap();
            if queue.len() > 1 {
                queue.remove(0)
            } else {
                queue[0].clone()
            }
        }
    }

    fn token(values: &[Option<&str>]) -> Arc<dyn TokenSource> {
        Arc::new(Token(StdMutex::new(values.iter().map(|v| v.map(str::to_string)).collect())))
    }

    /// A stand-in for Anthropic: answers with whatever status the test sets, counting hits.
    struct Endpoint {
        url: String,
        hits: Arc<AtomicUsize>,
        status: Arc<StdMutex<u16>>,
    }

    async fn endpoint(status: u16, body: &'static str, retry_after: Option<&'static str>) -> Endpoint {
        let hits = Arc::new(AtomicUsize::new(0));
        let current = Arc::new(StdMutex::new(status));
        let (h, s) = (hits.clone(), current.clone());
        let app = Router::new().route(
            "/usage",
            get(move || {
                let (h, s) = (h.clone(), s.clone());
                async move {
                    h.fetch_add(1, Ordering::SeqCst);
                    let code = *s.lock().unwrap();
                    let mut response = (StatusCode::from_u16(code).unwrap(), if code == 200 { body } else { "" }).into_response();
                    if let Some(retry) = retry_after {
                        if code == 429 {
                            response.headers_mut().insert("Retry-After", retry.parse().unwrap());
                        }
                    }
                    response
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Endpoint { url: format!("http://127.0.0.1:{port}/usage"), hits, status: current }
    }

    /// A clock the test moves by hand.
    #[derive(Clone)]
    struct Stepping(Arc<StdMutex<Instant>>);

    impl Stepping {
        fn new() -> Self {
            Self(Arc::new(StdMutex::new(Instant::parse("2026-09-15T10:00:00Z").unwrap())))
        }
        fn advance(&self, seconds: i64) {
            let mut now = self.0.lock().unwrap();
            *now = now.plus_seconds(seconds);
        }
        fn now(&self) -> Instant {
            *self.0.lock().unwrap()
        }
        fn clock(&self) -> rekall_service::Clock {
            let this = self.clone();
            Arc::new(move || this.now())
        }
    }

    #[tokio::test]
    async fn a_200_turns_each_present_window_into_a_row_and_grades_severity() {
        let endpoint = endpoint(200, SAMPLE, None).await;
        let view = ClaudeUsageService::new(token(&[Some("sk-token")]), &endpoint.url, rekall_service::system_clock()).current().await;
        assert_eq!(view.status, UsageStatus::Ok);
        let keys: Vec<&str> = view.limits.iter().map(|l| l.key.as_str()).collect();
        assert_eq!(keys, ["session", "weekly_all", "weekly_opus"]);
        assert_eq!(view.limits[0].percent, 80.0);
        assert_eq!(view.limits[0].severity, Severity::Warning);
        assert!(view.limits[0].resets_at.is_some());
        assert_eq!(view.limits[1].severity, Severity::Normal);
        assert_eq!(view.limits[2].severity, Severity::Critical);
    }

    #[tokio::test]
    async fn no_token_a_401_and_a_500_are_unauthenticated_unauthenticated_and_unavailable() {
        let ok = endpoint(200, SAMPLE, None).await;
        assert_eq!(ClaudeUsageService::new(token(&[None]), &ok.url, rekall_service::system_clock()).current().await.status, UsageStatus::Unauthenticated);
        let rejected = endpoint(401, "", None).await;
        assert_eq!(ClaudeUsageService::new(token(&[Some("stale")]), &rejected.url, rekall_service::system_clock()).current().await.status, UsageStatus::Unauthenticated);
        let down = endpoint(500, "", None).await;
        assert_eq!(ClaudeUsageService::new(token(&[Some("sk")]), &down.url, rekall_service::system_clock()).current().await.status, UsageStatus::Unavailable);
    }

    #[tokio::test]
    async fn a_429_is_obeyed_until_retry_after_has_passed() {
        let clock = Stepping::new();
        let endpoint = endpoint(429, "", Some("120")).await;
        let service = ClaudeUsageService::new(token(&[Some("sk")]), &endpoint.url, clock.clock());
        let limited = service.current().await;
        assert_eq!(limited.status, UsageStatus::RateLimited);
        assert_eq!(limited.retry_at, Some(clock.now().plus_seconds(120)));
        clock.advance(119);
        assert_eq!(service.refresh().await.status, UsageStatus::RateLimited);
        assert_eq!(service.current().await.status, UsageStatus::RateLimited);
        assert_eq!(endpoint.hits.load(Ordering::SeqCst), 1);
        clock.advance(2);
        service.current().await;
        assert_eq!(endpoint.hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn a_429_with_no_retry_after_holds_for_five_minutes() {
        let clock = Stepping::new();
        let endpoint = endpoint(429, "", None).await;
        let service = ClaudeUsageService::new(token(&[Some("sk")]), &endpoint.url, clock.clock());
        assert_eq!(service.current().await.retry_at, Some(clock.now().plus_minutes(5)));
        clock.advance(240);
        service.refresh().await;
        assert_eq!(endpoint.hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_rate_limit_after_a_good_read_keeps_the_figures_and_says_when_they_refresh() {
        let clock = Stepping::new();
        let endpoint = endpoint(200, SAMPLE, Some("60")).await;
        let service = ClaudeUsageService::new(token(&[Some("sk")]), &endpoint.url, clock.clock());
        assert_eq!(service.current().await.status, UsageStatus::Ok);
        *endpoint.status.lock().unwrap() = 429;
        let held = service.refresh().await;
        assert_eq!(held.status, UsageStatus::Ok);
        assert!(!held.limits.is_empty());
        assert_eq!(held.retry_at, Some(clock.now().plus_seconds(60)));
        assert!(service.refresh().await.retry_at.is_some());
        assert_eq!(endpoint.hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn the_cache_holds_a_good_read_for_a_minute_and_a_refresh_skips_it() {
        let endpoint = endpoint(200, SAMPLE, None).await;
        let service = ClaudeUsageService::new(token(&[Some("sk")]), &endpoint.url, rekall_service::system_clock());
        service.current().await;
        service.current().await;
        assert_eq!(endpoint.hits.load(Ordering::SeqCst), 1);
        assert_eq!(service.refresh().await.status, UsageStatus::Ok);
        assert_eq!(endpoint.hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn a_degraded_reading_is_retried_after_ten_seconds() {
        let clock = Stepping::new();
        let endpoint = endpoint(500, "", None).await;
        let service = ClaudeUsageService::new(token(&[Some("sk")]), &endpoint.url, clock.clock());
        assert_eq!(service.current().await.status, UsageStatus::Unavailable);
        clock.advance(9);
        service.current().await;
        assert_eq!(endpoint.hits.load(Ordering::SeqCst), 1);
        clock.advance(2);
        service.current().await;
        assert_eq!(endpoint.hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn a_missing_token_at_launch_is_not_remembered_once_the_next_poll_finds_one() {
        let clock = Stepping::new();
        let endpoint = endpoint(200, SAMPLE, None).await;
        let service = ClaudeUsageService::new(token(&[None, Some("sk")]), &endpoint.url, clock.clock());
        assert_eq!(service.current().await.status, UsageStatus::Unauthenticated);
        clock.advance(11);
        assert_eq!(service.current().await.status, UsageStatus::Ok);
        assert_eq!(endpoint.hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_poll_in_flight_at_shutdown_is_cut_loose() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = Router::new().route("/usage", get(|| async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            ""
        }));
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let service = Arc::new(ClaudeUsageService::new(token(&[Some("sk")]), &format!("http://127.0.0.1:{port}/usage"), rekall_service::system_clock()));
        let polling = service.clone();
        let poll = tokio::spawn(async move { polling.current().await });
        tokio::time::sleep(Duration::from_millis(300)).await;
        service.release_on_shutdown();
        let view = tokio::time::timeout(Duration::from_secs(3), poll).await.expect("cut loose").unwrap();
        assert_eq!(view.status, UsageStatus::Unavailable);
    }

    #[tokio::test]
    async fn after_shutdown_no_further_call_reaches_the_endpoint() {
        let endpoint = endpoint(200, SAMPLE, None).await;
        let service = ClaudeUsageService::new(token(&[Some("sk")]), &endpoint.url, rekall_service::system_clock());
        service.current().await;
        service.release_on_shutdown();
        let after = service.current().await;
        assert_eq!(endpoint.hits.load(Ordering::SeqCst), 1);
        assert_eq!(after.status, UsageStatus::Ok);
    }
}
