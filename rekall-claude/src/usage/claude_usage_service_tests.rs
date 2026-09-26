use std::sync::atomic::AtomicUsize;
use std::sync::Mutex as StdMutex;

use axum::http::StatusCode;
use axum::response::IntoResponse;

use axum::routing::get;
use axum::Router;

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
