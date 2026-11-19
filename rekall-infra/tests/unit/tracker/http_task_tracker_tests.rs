use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode as AxumStatus};
use axum::routing::{delete, get, patch, post, put};
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
                assert_eq!(body, json!({ "title": "Fix login", "description": "It fails.\n\nOn Safari.", "assignee_ids": [42], "issue_type": "task" }));
                Json(json!({ "iid": 7, "web_url": "https://gitlab.example.com/acme/platform/-/issues/7" }))
            }),
        )
        .route("/api/v4/user", get(|| async { Json(json!({ "id": 42, "username": "ada" })) }))
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
                assert_eq!(body, json!({ "title": "Fix login", "body": "It fails.\n\nOn Safari.", "assignees": ["ada"], "type": "Task" }));
                Json(json!({ "number": 12, "html_url": "https://github.example.com/acme/app/issues/12" }))
            }),
        )
        .route("/api/v3/user", get(|| async { Json(json!({ "login": "ada" })) }))
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

#[tokio::test]
async fn a_jira_cloud_issue_is_assigned_to_the_account_of_the_credential() {
    let router = Router::new()
        .route(
            "/rest/api/3/issue",
            post(|Json(body): Json<Value>| async move {
                assert_eq!(body["fields"]["assignee"], json!({ "accountId": "acc-1" }));
                Json(json!({ "key": "ENG-5" }))
            }),
        )
        .route("/rest/api/3/myself", get(|| async { Json(json!({ "accountId": "acc-1" })) }));
    let site = serve(router).await;

    let link = HttpTaskTracker::new().create(&endpoint(ConnectionKind::Jira, &site, "ENG", "ada@example.com:atl-good"), &issue()).await.unwrap();

    assert_eq!(link.key, "ENG-5");
}

#[tokio::test]
async fn an_issue_is_created_unassigned_when_the_tracker_does_not_say_who_the_credential_is() {
    let router = Router::new().route(
        "/api/v3/repos/acme/app/issues",
        post(|Json(body): Json<Value>| async move {
            assert!(body.get("assignees").is_none());
            Json(json!({ "number": 13, "html_url": "https://github.example.com/acme/app/issues/13" }))
        }),
    );
    let site = serve(router).await;

    let link = HttpTaskTracker::new().create(&endpoint(ConnectionKind::Github, &site, "acme/app", "ghp_secret"), &issue()).await.unwrap();

    assert_eq!(link.key, "13");
}

#[tokio::test]
async fn a_repository_without_issue_types_gets_the_issue_without_one() {
    let router = Router::new().route(
        "/api/v3/repos/acme/app/issues",
        post(|Json(body): Json<Value>| async move {
            if body.get("type").is_some() {
                return (AxumStatus::UNPROCESSABLE_ENTITY, Json(json!({ "message": "Validation Failed" })));
            }
            (AxumStatus::CREATED, Json(json!({ "number": 14, "html_url": "https://github.example.com/acme/app/issues/14" })))
        }),
    );
    let site = serve(router).await;

    let link = HttpTaskTracker::new().create(&endpoint(ConnectionKind::Github, &site, "acme/app", "ghp_secret"), &issue()).await.unwrap();

    assert_eq!(link.key, "14");
}

#[tokio::test]
async fn a_gitlab_and_a_jira_issue_are_deleted_by_their_key() {
    let router = Router::new()
        .route("/api/v4/projects/{project}/issues/{iid}", delete(|Path((project, iid)): Path<(String, String)>| async move {
            assert_eq!((project.as_str(), iid.as_str()), ("acme/platform", "7"));
            AxumStatus::NO_CONTENT
        }))
        .route("/rest/api/3/issue/ENG-3", delete(|| async { AxumStatus::NO_CONTENT }));
    let site = serve(router).await;
    let tracker = HttpTaskTracker::new();

    tracker.delete(&endpoint(ConnectionKind::Gitlab, &site, "acme/platform", "glpat-secret"), "7").await.unwrap();
    tracker.delete(&endpoint(ConnectionKind::Jira, &site, "ENG", "ada@example.com:atl-good"), "ENG-3").await.unwrap();
}

fn github_deleting(graphql_answer: Value) -> Router {
    Router::new()
        .route("/api/v3/repos/acme/app/issues/12", get(|| async { Json(json!({ "node_id": "I_kw1" })) }).patch(|Json(body): Json<Value>| async move {
            assert_eq!(body, json!({ "state": "closed", "state_reason": "not_planned" }));
            Json(json!({}))
        }))
        .route("/api/graphql", post(move |Json(body): Json<Value>| {
            let answer = graphql_answer.clone();
            async move {
                assert_eq!(body["variables"]["id"], "I_kw1");
                Json(answer)
            }
        }))
}

#[tokio::test]
async fn a_github_issue_is_deleted_through_its_graphql_api() {
    let site = serve(github_deleting(json!({ "data": { "deleteIssue": { "clientMutationId": null } } }))).await;

    HttpTaskTracker::new().delete(&endpoint(ConnectionKind::Github, &site, "acme/app", "ghp_secret"), "12").await.unwrap();
}

#[tokio::test]
async fn a_github_issue_that_cannot_be_deleted_is_closed_as_not_planned() {
    let site = serve(github_deleting(json!({ "errors": [{ "message": "Resource not accessible by personal access token" }] }))).await;

    HttpTaskTracker::new().delete(&endpoint(ConnectionKind::Github, &site, "acme/app", "ghp_secret"), "12").await.unwrap();
}
