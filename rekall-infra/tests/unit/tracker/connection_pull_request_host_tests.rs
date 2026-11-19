use axum::routing::{get, put};
use axum::{Json, Router};
use rekall_core::connection::{Mergeability, Secret};
use serde_json::json;

use super::*;

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

fn endpoint(kind: ConnectionKind, site_url: &str) -> TrackerEndpoint {
    TrackerEndpoint { kind, site_url: site_url.into(), proxy_url: None, location: "acme/app".into(), credential: Secret::new("secret") }
}

#[tokio::test]
async fn a_github_connection_is_served_by_the_github_api() {
    let site = serve(Router::new().route("/api/v3/repos/acme/app/pulls", get(|| async { Json(json!([])) }))).await;

    let found = ConnectionPullRequestHost::new().find(&endpoint(ConnectionKind::Github, &site), "rekall/fix-login").await.unwrap();

    assert_eq!(found, None);
}

#[tokio::test]
async fn a_gitlab_connection_is_served_by_the_gitlab_api() {
    let router = Router::new().route(
        "/api/v4/projects/acme%2Fapp/merge_requests/5/merge",
        put(|| async { Json(json!({ "iid": 5, "web_url": "x", "merge_status": "can_be_merged" })) }),
    );
    let site = serve(router).await;
    let merge_request = PullRequest { number: 5, url: "x".into(), mergeability: Mergeability::Mergeable };

    let merged = ConnectionPullRequestHost::new().merge(&endpoint(ConnectionKind::Gitlab, &site), &merge_request).await.unwrap();

    assert_eq!(merged, PullRequestMerge::Merged);
}

#[tokio::test]
async fn a_jira_connection_takes_no_pull_request() {
    let failure = ConnectionPullRequestHost::new().find(&endpoint(ConnectionKind::Jira, "https://jira.example.com"), "rekall/fix-login").await.unwrap_err();

    assert!(matches!(failure, TrackerFailure::Failed { .. }));
}
