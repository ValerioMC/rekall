use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode as AxumStatus};
use axum::routing::{patch, post, put};
use axum::{Json, Router};
use rekall_core::connection::Secret;

use super::*;

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

fn issue() -> TrackerIssue {
    TrackerIssue { title: "Fix login".into(), description: "It fails.\n\nOn Safari.".into() }
}

fn endpoint(kind: ConnectionKind, site_url: &str, location: &str, credential: &str) -> TrackerEndpoint {
    TrackerEndpoint { kind, site_url: site_url.into(), proxy_url: None, location: location.into(), credential: Secret::new(credential) }
}

fn header(headers: &HeaderMap, name: &str) -> String {
    headers.get(name).and_then(|value| value.to_str().ok()).unwrap_or_default().to_string()
}

#[tokio::test]
async fn a_gitlab_issue_is_created_and_rewritten_in_the_project_named_by_its_encoded_path() {
    let router = Router::new()
        .route(
            "/api/v4/projects/{project}/issues",
            post(|Path(project): Path<String>, headers: HeaderMap, Json(body): Json<Value>| async move {
                assert_eq!(project, "acme/platform");
                assert_eq!(header(&headers, "private-token"), "glpat-secret");
                assert_eq!(body, json!({ "title": "Fix login", "description": "It fails.\n\nOn Safari." }));
                Json(json!({ "iid": 7, "web_url": "https://gitlab.example.com/acme/platform/-/issues/7" }))
            }),
        )
        .route(
            "/api/v4/projects/{project}/issues/{iid}",
            put(|Path((project, iid)): Path<(String, String)>, Json(body): Json<Value>| async move {
                assert_eq!((project.as_str(), iid.as_str()), ("acme/platform", "7"));
                assert_eq!(body["title"], "Fix login");
                Json(json!({}))
            }),
        );
    let site = serve(router).await;
    let endpoint = endpoint(ConnectionKind::Gitlab, &site, "acme/platform", "glpat-secret");
    let tracker = HttpTaskTracker::new();

    let link = tracker.create(&endpoint, &issue()).await.unwrap();
    tracker.update(&endpoint, &link.key, &issue()).await.unwrap();

    assert_eq!(link, TrackerLink { key: "7".into(), url: "https://gitlab.example.com/acme/platform/-/issues/7".into() });
}

#[tokio::test]
async fn a_github_issue_is_created_and_patched_through_the_enterprise_api_root() {
    let router = Router::new()
        .route(
            "/api/v3/repos/acme/app/issues",
            post(|headers: HeaderMap, Json(body): Json<Value>| async move {
                assert_eq!(header(&headers, "authorization"), "Bearer ghp_secret");
                assert_eq!(body, json!({ "title": "Fix login", "body": "It fails.\n\nOn Safari." }));
                Json(json!({ "number": 12, "html_url": "https://github.example.com/acme/app/issues/12" }))
            }),
        )
        .route("/api/v3/repos/acme/app/issues/12", patch(|Json(body): Json<Value>| async move { Json(body) }));
    let site = serve(router).await;
    let endpoint = endpoint(ConnectionKind::Github, &site, "acme/app", "ghp_secret");
    let tracker = HttpTaskTracker::new();

    let link = tracker.create(&endpoint, &issue()).await.unwrap();
    tracker.update(&endpoint, "12", &issue()).await.unwrap();

    assert_eq!(link, TrackerLink { key: "12".into(), url: "https://github.example.com/acme/app/issues/12".into() });
}

#[tokio::test]
async fn a_jira_cloud_issue_is_written_with_a_document_description_and_links_to_its_browse_page() {
    let router = Router::new()
        .route(
            "/rest/api/3/issue",
            post(|headers: HeaderMap, Json(body): Json<Value>| async move {
                assert_eq!(header(&headers, "authorization"), "Basic YWRhQGV4YW1wbGUuY29tOmF0bC1nb29k");
                assert_eq!(body["fields"]["project"]["key"], "ENG");
                assert_eq!(body["fields"]["summary"], "Fix login");
                assert_eq!(body["fields"]["issuetype"]["name"], "Task");
                assert_eq!(body["fields"]["description"]["content"].as_array().unwrap().len(), 2);
                Json(json!({ "key": "ENG-3" }))
            }),
        )
        .route(
            "/rest/api/3/issue/ENG-3",
            put(|Json(body): Json<Value>| async move {
                assert_eq!(body["fields"]["summary"], "Fix login");
                AxumStatus::NO_CONTENT
            }),
        );
    let site = serve(router).await;
    let endpoint = endpoint(ConnectionKind::Jira, &site, "ENG", "ada@example.com:atl-good");
    let tracker = HttpTaskTracker::new();

    let link = tracker.create(&endpoint, &issue()).await.unwrap();
    tracker.update(&endpoint, &link.key, &issue()).await.unwrap();

    assert_eq!(link, TrackerLink { key: "ENG-3".into(), url: format!("{site}/browse/ENG-3") });
}

#[tokio::test]
async fn a_jira_data_center_issue_keeps_its_description_as_text() {
    let router = Router::new().route(
        "/rest/api/2/issue",
        post(|headers: HeaderMap, Json(body): Json<Value>| async move {
            assert_eq!(header(&headers, "authorization"), "Bearer dc-token");
            assert_eq!(body["fields"]["description"], "It fails.\n\nOn Safari.");
            Json(json!({ "key": "ENG-4" }))
        }),
    );
    let site = serve(router).await;

    let link = HttpTaskTracker::new().create(&endpoint(ConnectionKind::Jira, &site, "ENG", "pat:dc-token"), &issue()).await.unwrap();

    assert_eq!(link.key, "ENG-4");
}

#[tokio::test]
async fn a_credential_the_site_refuses_is_rejected_and_other_refusals_say_why() {
    let router = Router::new()
        .route("/api/v4/projects/{project}/issues", post(|| async { AxumStatus::UNAUTHORIZED }))
        .route("/api/v3/repos/acme/app/issues", post(|| async { (AxumStatus::UNPROCESSABLE_ENTITY, Json(json!({ "message": "Validation Failed" }))) }));
    let site = serve(router).await;
    let tracker = HttpTaskTracker::new();

    let rejected = tracker.create(&endpoint(ConnectionKind::Gitlab, &site, "acme/platform", "bad"), &issue()).await.unwrap_err();
    let refused = tracker.create(&endpoint(ConnectionKind::Github, &site, "acme/app", "good"), &issue()).await.unwrap_err();

    assert_eq!(rejected, TrackerFailure::Rejected { status: 401 });
    assert_eq!(refused, TrackerFailure::Failed { reason: "it answered HTTP 422: Validation Failed".into() });
}

#[tokio::test]
async fn a_cluster_is_no_task_manager() {
    let failure = HttpTaskTracker::new().create(&endpoint(ConnectionKind::Kubernetes, "https://k8s.example.com", "apps", "token"), &issue()).await.unwrap_err();

    assert!(matches!(failure, TrackerFailure::Failed { .. }));
}
