use std::collections::HashMap;

use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode as AxumStatus};
use axum::routing::{delete, get, patch, put};
use axum::{Json, Router};
use rekall_core::connection::Secret;

use super::*;

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

fn endpoint(site_url: &str) -> TrackerEndpoint {
    TrackerEndpoint { kind: ConnectionKind::Github, site_url: site_url.into(), proxy_url: None, location: "acme/app".into(), credential: Secret::new("ghp_secret") }
}

fn pull(number: u64, mergeable: Value, state: &str) -> Value {
    json!({ "number": number, "html_url": format!("https://github.example.com/acme/app/pull/{number}"), "mergeable": mergeable, "mergeable_state": state })
}

fn mergeable_pull_request(number: u64) -> PullRequest {
    PullRequest { number, url: format!("https://github.example.com/acme/app/pull/{number}"), mergeability: Mergeability::Mergeable }
}

fn header(headers: &HeaderMap, name: &str) -> String {
    headers.get(name).and_then(|value| value.to_str().ok()).unwrap_or_default().to_string()
}

#[tokio::test]
async fn a_pull_request_is_opened_from_the_branch_against_the_base_when_none_is_open() {
    let router = Router::new().route(
        "/api/v3/repos/acme/app/pulls",
        get(|Query(query): Query<HashMap<String, String>>| async move {
            assert_eq!(query["head"], "acme:rekall/fix-login");
            assert_eq!(query["state"], "open");
            Json(json!([]))
        })
        .post(|headers: HeaderMap, Json(body): Json<Value>| async move {
            assert_eq!(header(&headers, "authorization"), "Bearer ghp_secret");
            assert_eq!(body, json!({ "title": "Fix login", "head": "rekall/fix-login", "base": "main", "body": "Closes #12" }));
            Json(pull(5, Value::Null, "unknown"))
        }),
    );
    let site = serve(router).await;

    let opened = GithubPullRequestHost::new().open(&endpoint(&site), "rekall/fix-login", "main", "Fix login", "Closes #12").await.unwrap();

    assert_eq!(opened, PullRequest { number: 5, url: "https://github.example.com/acme/app/pull/5".into(), mergeability: Mergeability::Unknown });
}

#[tokio::test]
async fn opening_again_returns_the_pull_request_already_open_for_the_branch() {
    let router = Router::new()
        .route("/api/v3/repos/acme/app/pulls", get(|| async { Json(json!([{ "number": 5, "html_url": "x" }])) }).post(|| async { AxumStatus::INTERNAL_SERVER_ERROR }))
        .route("/api/v3/repos/acme/app/pulls/5", get(|| async { Json(pull(5, json!(true), "clean")) }));
    let site = serve(router).await;

    let opened = GithubPullRequestHost::new().open(&endpoint(&site), "rekall/fix-login", "main", "Fix login", "").await.unwrap();

    assert_eq!(opened, mergeable_pull_request(5));
}

#[tokio::test]
async fn a_pull_request_that_cannot_merge_cleanly_is_found_as_conflicting() {
    let router = Router::new()
        .route("/api/v3/repos/acme/app/pulls", get(|| async { Json(json!([{ "number": 6, "html_url": "x" }])) }))
        .route("/api/v3/repos/acme/app/pulls/6", get(|| async { Json(pull(6, json!(false), "dirty")) }));
    let site = serve(router).await;

    let found = GithubPullRequestHost::new().find(&endpoint(&site), "rekall/fix-login").await.unwrap().unwrap();

    assert_eq!(found.mergeability, Mergeability::Conflicting);
}

#[tokio::test]
async fn no_open_pull_request_for_the_branch_is_none() {
    let site = serve(Router::new().route("/api/v3/repos/acme/app/pulls", get(|| async { Json(json!([])) }))).await;

    let found = GithubPullRequestHost::new().find(&endpoint(&site), "rekall/fix-login").await.unwrap();

    assert_eq!(found, None);
}

#[tokio::test]
async fn a_pull_request_is_merged_with_a_merge_commit() {
    let router = Router::new().route(
        "/api/v3/repos/acme/app/pulls/5/merge",
        put(|Json(body): Json<Value>| async move {
            assert_eq!(body, json!({ "merge_method": "merge" }));
            Json(json!({ "merged": true }))
        }),
    );
    let site = serve(router).await;

    let merged = GithubPullRequestHost::new().merge(&endpoint(&site), &mergeable_pull_request(5)).await.unwrap();

    assert_eq!(merged, PullRequestMerge::Merged);
}

#[tokio::test]
async fn a_merge_github_turns_down_over_conflicts_is_conflicting() {
    let router = Router::new()
        .route("/api/v3/repos/acme/app/pulls/5/merge", put(|| async { (AxumStatus::METHOD_NOT_ALLOWED, Json(json!({ "message": "Pull Request is not mergeable" }))) }))
        .route("/api/v3/repos/acme/app/pulls/5", get(|| async { Json(pull(5, json!(false), "dirty")) }));
    let site = serve(router).await;

    let merged = GithubPullRequestHost::new().merge(&endpoint(&site), &mergeable_pull_request(5)).await.unwrap();

    assert_eq!(merged, PullRequestMerge::Conflicting);
}

#[tokio::test]
async fn a_merge_github_turns_down_for_another_reason_is_refused_with_its_message() {
    let router = Router::new()
        .route("/api/v3/repos/acme/app/pulls/5/merge", put(|| async { (AxumStatus::METHOD_NOT_ALLOWED, Json(json!({ "message": "Merge commits are not allowed on this repository." }))) }))
        .route("/api/v3/repos/acme/app/pulls/5", get(|| async { Json(pull(5, json!(true), "clean")) }));
    let site = serve(router).await;

    let merged = GithubPullRequestHost::new().merge(&endpoint(&site), &mergeable_pull_request(5)).await.unwrap();

    assert_eq!(merged, PullRequestMerge::Refused("Merge commits are not allowed on this repository.".into()));
}

#[tokio::test]
async fn a_server_error_on_merge_is_a_failure_to_retry_not_a_refusal() {
    let site = serve(Router::new().route("/api/v3/repos/acme/app/pulls/5/merge", put(|| async { AxumStatus::BAD_GATEWAY }))).await;

    let failure = GithubPullRequestHost::new().merge(&endpoint(&site), &mergeable_pull_request(5)).await.unwrap_err();

    assert_eq!(failure, TrackerFailure::Failed { reason: "it answered HTTP 502".into() });
}

#[tokio::test]
async fn a_credential_github_refuses_is_rejected() {
    let site = serve(Router::new().route("/api/v3/repos/acme/app/pulls/5/merge", put(|| async { AxumStatus::UNAUTHORIZED }))).await;

    let failure = GithubPullRequestHost::new().merge(&endpoint(&site), &mergeable_pull_request(5)).await.unwrap_err();

    assert_eq!(failure, TrackerFailure::Rejected { status: 401 });
}

#[tokio::test]
async fn an_issue_is_closed_by_patching_its_state() {
    let router = Router::new().route(
        "/api/v3/repos/acme/app/issues/12",
        patch(|Json(body): Json<Value>| async move {
            assert_eq!(body, json!({ "state": "closed" }));
            Json(json!({}))
        }),
    );
    let site = serve(router).await;

    GithubPullRequestHost::new().close_issue(&endpoint(&site), "12").await.unwrap();
}

#[tokio::test]
async fn a_branch_is_deleted_by_deleting_its_ref() {
    let router = Router::new().route("/api/v3/repos/acme/app/git/refs/heads/rekall/fix-login", delete(|| async { AxumStatus::NO_CONTENT }));
    let site = serve(router).await;

    GithubPullRequestHost::new().delete_branch(&endpoint(&site), "rekall/fix-login").await.unwrap();
}

#[tokio::test]
async fn a_branch_the_host_no_longer_has_is_already_deleted() {
    let router = Router::new().route("/api/v3/repos/acme/app/git/refs/heads/rekall/fix-login", delete(|| async { AxumStatus::UNPROCESSABLE_ENTITY }));
    let site = serve(router).await;

    GithubPullRequestHost::new().delete_branch(&endpoint(&site), "rekall/fix-login").await.unwrap();
}

#[tokio::test]
async fn a_connection_that_is_not_github_takes_no_pull_request() {
    let gitlab = TrackerEndpoint { kind: ConnectionKind::Gitlab, ..endpoint("https://gitlab.example.com") };

    let failure = GithubPullRequestHost::new().find(&gitlab, "rekall/fix-login").await.unwrap_err();

    assert!(matches!(failure, TrackerFailure::Failed { .. }));
}
