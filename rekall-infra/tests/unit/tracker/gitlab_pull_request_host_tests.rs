use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode as AxumStatus};
use axum::routing::{delete, get, put};
use axum::{Json, Router};
use rekall_core::connection::Secret;

use super::*;

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

fn host() -> GitlabPullRequestHost {
    GitlabPullRequestHost { timeout: TIMEOUT, check_interval: Duration::from_millis(1) }
}

fn endpoint(site_url: &str) -> TrackerEndpoint {
    TrackerEndpoint { kind: ConnectionKind::Gitlab, site_url: site_url.into(), proxy_url: None, location: "acme/app".into(), credential: Secret::new("glpat-secret") }
}

fn merge_request(iid: u64, merge_status: &str) -> Value {
    json!({ "iid": iid, "web_url": format!("https://gitlab.example.com/acme/app/-/merge_requests/{iid}"), "merge_status": merge_status })
}

fn mergeable_pull_request(iid: u64) -> PullRequest {
    PullRequest { number: iid, url: format!("https://gitlab.example.com/acme/app/-/merge_requests/{iid}"), mergeability: Mergeability::Mergeable }
}

fn header(headers: &HeaderMap, name: &str) -> String {
    headers.get(name).and_then(|value| value.to_str().ok()).unwrap_or_default().to_string()
}

#[tokio::test]
async fn a_merge_request_is_opened_from_the_branch_against_the_base_when_none_is_open() {
    let router = Router::new().route(
        "/api/v4/projects/acme%2Fapp/merge_requests",
        get(|Query(query): Query<HashMap<String, String>>| async move {
            assert_eq!(query["source_branch"], "rekall/fix-login");
            assert_eq!(query["state"], "opened");
            Json(json!([]))
        })
        .post(|headers: HeaderMap, Json(body): Json<Value>| async move {
            assert_eq!(header(&headers, "private-token"), "glpat-secret");
            assert_eq!(body, json!({ "source_branch": "rekall/fix-login", "target_branch": "main", "title": "Fix login", "description": "Closes #12" }));
            Json(merge_request(5, "checking"))
        }),
    );
    let site = serve(router).await;

    let opened = host().open(&endpoint(&site), "rekall/fix-login", "main", "Fix login", "Closes #12").await.unwrap();

    assert_eq!(opened, PullRequest { number: 5, url: "https://gitlab.example.com/acme/app/-/merge_requests/5".into(), mergeability: Mergeability::Unknown });
}

#[tokio::test]
async fn opening_again_returns_the_merge_request_already_open_for_the_branch() {
    let router = Router::new()
        .route("/api/v4/projects/acme%2Fapp/merge_requests", get(|| async { Json(json!([merge_request(5, "unchecked")])) }).post(|| async { AxumStatus::INTERNAL_SERVER_ERROR }))
        .route("/api/v4/projects/acme%2Fapp/merge_requests/5", get(|| async { Json(merge_request(5, "can_be_merged")) }));
    let site = serve(router).await;

    let opened = host().open(&endpoint(&site), "rekall/fix-login", "main", "Fix login", "").await.unwrap();

    assert_eq!(opened, mergeable_pull_request(5));
}

#[tokio::test]
async fn a_merge_request_that_cannot_be_merged_is_found_as_conflicting() {
    let router = Router::new()
        .route("/api/v4/projects/acme%2Fapp/merge_requests", get(|| async { Json(json!([merge_request(6, "cannot_be_merged")])) }))
        .route("/api/v4/projects/acme%2Fapp/merge_requests/6", get(|| async { Json(merge_request(6, "cannot_be_merged")) }));
    let site = serve(router).await;

    let found = host().find(&endpoint(&site), "rekall/fix-login").await.unwrap().unwrap();

    assert_eq!(found.mergeability, Mergeability::Conflicting);
}

#[tokio::test]
async fn a_merge_request_still_being_checked_is_read_again_until_gitlab_knows() {
    let reads = Arc::new(AtomicUsize::new(0));
    let router = Router::new()
        .route("/api/v4/projects/acme%2Fapp/merge_requests", get(|| async { Json(json!([merge_request(7, "checking")])) }))
        .route(
            "/api/v4/projects/acme%2Fapp/merge_requests/7",
            get(|State(reads): State<Arc<AtomicUsize>>| async move {
                let status = if reads.fetch_add(1, Ordering::SeqCst) < 2 { "checking" } else { "can_be_merged" };
                Json(merge_request(7, status))
            }),
        )
        .with_state(reads.clone());
    let site = serve(router).await;

    let found = host().find(&endpoint(&site), "rekall/fix-login").await.unwrap().unwrap();

    assert_eq!(found.mergeability, Mergeability::Mergeable);
    assert_eq!(reads.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn a_merge_request_that_stays_unchecked_is_unknown_after_a_few_reads() {
    let reads = Arc::new(AtomicUsize::new(0));
    let router = Router::new()
        .route("/api/v4/projects/acme%2Fapp/merge_requests", get(|| async { Json(json!([merge_request(7, "checking")])) }))
        .route(
            "/api/v4/projects/acme%2Fapp/merge_requests/7",
            get(|State(reads): State<Arc<AtomicUsize>>| async move {
                reads.fetch_add(1, Ordering::SeqCst);
                Json(merge_request(7, "checking"))
            }),
        )
        .with_state(reads.clone());
    let site = serve(router).await;

    let found = host().find(&endpoint(&site), "rekall/fix-login").await.unwrap().unwrap();

    assert_eq!(found.mergeability, Mergeability::Unknown);
    assert_eq!(reads.load(Ordering::SeqCst), CHECK_ATTEMPTS);
}

#[tokio::test]
async fn no_open_merge_request_for_the_branch_is_none() {
    let site = serve(Router::new().route("/api/v4/projects/acme%2Fapp/merge_requests", get(|| async { Json(json!([])) }))).await;

    let found = host().find(&endpoint(&site), "rekall/fix-login").await.unwrap();

    assert_eq!(found, None);
}

#[tokio::test]
async fn a_merge_request_is_merged_without_squashing() {
    let router = Router::new().route(
        "/api/v4/projects/acme%2Fapp/merge_requests/5/merge",
        put(|Json(body): Json<Value>| async move {
            assert_eq!(body, json!({ "squash": false }));
            Json(merge_request(5, "can_be_merged"))
        }),
    );
    let site = serve(router).await;

    let merged = host().merge(&endpoint(&site), &mergeable_pull_request(5)).await.unwrap();

    assert_eq!(merged, PullRequestMerge::Merged);
}

#[tokio::test]
async fn a_merge_gitlab_turns_down_over_conflicts_is_conflicting() {
    let router = Router::new()
        .route("/api/v4/projects/acme%2Fapp/merge_requests/5/merge", put(|| async { (AxumStatus::NOT_ACCEPTABLE, Json(json!({ "message": "Branch cannot be merged" }))) }))
        .route("/api/v4/projects/acme%2Fapp/merge_requests/5", get(|| async { Json(merge_request(5, "cannot_be_merged")) }));
    let site = serve(router).await;

    let merged = host().merge(&endpoint(&site), &mergeable_pull_request(5)).await.unwrap();

    assert_eq!(merged, PullRequestMerge::Conflicting);
}

#[tokio::test]
async fn a_merge_gitlab_turns_down_for_another_reason_is_refused_with_its_message() {
    let router = Router::new()
        .route("/api/v4/projects/acme%2Fapp/merge_requests/5/merge", put(|| async { (AxumStatus::METHOD_NOT_ALLOWED, Json(json!({ "message": "405 Method Not Allowed" }))) }))
        .route("/api/v4/projects/acme%2Fapp/merge_requests/5", get(|| async { Json(merge_request(5, "can_be_merged")) }));
    let site = serve(router).await;

    let merged = host().merge(&endpoint(&site), &mergeable_pull_request(5)).await.unwrap();

    assert_eq!(merged, PullRequestMerge::Refused("405 Method Not Allowed".into()));
}

#[tokio::test]
async fn a_server_error_on_merge_is_a_failure_to_retry_not_a_refusal() {
    let site = serve(Router::new().route("/api/v4/projects/acme%2Fapp/merge_requests/5/merge", put(|| async { AxumStatus::BAD_GATEWAY }))).await;

    let failure = host().merge(&endpoint(&site), &mergeable_pull_request(5)).await.unwrap_err();

    assert_eq!(failure, TrackerFailure::Failed { reason: "it answered HTTP 502".into() });
}

#[tokio::test]
async fn a_credential_gitlab_refuses_is_rejected() {
    let site = serve(Router::new().route("/api/v4/projects/acme%2Fapp/merge_requests/5/merge", put(|| async { AxumStatus::UNAUTHORIZED }))).await;

    let failure = host().merge(&endpoint(&site), &mergeable_pull_request(5)).await.unwrap_err();

    assert_eq!(failure, TrackerFailure::Rejected { status: 401 });
}

#[tokio::test]
async fn an_issue_is_closed_by_a_close_state_event() {
    let router = Router::new().route(
        "/api/v4/projects/acme%2Fapp/issues/12",
        put(|Json(body): Json<Value>| async move {
            assert_eq!(body, json!({ "state_event": "close" }));
            Json(json!({}))
        }),
    );
    let site = serve(router).await;

    host().close_issue(&endpoint(&site), "12").await.unwrap();
}

#[tokio::test]
async fn a_branch_is_deleted_by_its_encoded_name() {
    let router = Router::new().route("/api/v4/projects/acme%2Fapp/repository/branches/rekall%2Ffix-login", delete(|| async { AxumStatus::NO_CONTENT }));
    let site = serve(router).await;

    host().delete_branch(&endpoint(&site), "rekall/fix-login").await.unwrap();
}

#[tokio::test]
async fn a_branch_gitlab_no_longer_has_is_already_deleted() {
    let router = Router::new().route("/api/v4/projects/acme%2Fapp/repository/branches/rekall%2Ffix-login", delete(|| async { AxumStatus::NOT_FOUND }));
    let site = serve(router).await;

    host().delete_branch(&endpoint(&site), "rekall/fix-login").await.unwrap();
}

#[tokio::test]
async fn a_connection_that_is_not_gitlab_takes_no_merge_request() {
    let github = TrackerEndpoint { kind: ConnectionKind::Github, ..endpoint("https://github.example.com") };

    let failure = host().find(&github, "rekall/fix-login").await.unwrap_err();

    assert!(matches!(failure, TrackerFailure::Failed { .. }));
}
