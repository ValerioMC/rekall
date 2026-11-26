use axum::body::Bytes;
use axum::extract::{Multipart, Path, Query};
use axum::http::{HeaderMap, StatusCode as AxumStatus};
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_core::connection::Secret;
use serde_json::{json, Value};
use std::collections::HashMap;

use super::*;

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

fn endpoint(kind: ConnectionKind, site_url: &str, location: &str, credential: &str) -> TrackerEndpoint {
    TrackerEndpoint { kind, site_url: site_url.into(), proxy_url: None, location: location.into(), credential: Secret::new(credential) }
}

fn picture() -> PicturePath {
    PicturePath::parse("acme/app/app-1/login-3f9a1c2b.svg").unwrap()
}

async fn upload(endpoint: &TrackerEndpoint, issue_key: Option<&str>, github_uploads: Option<&str>) -> Result<String, TrackerFailure> {
    PictureUpload { endpoint, picture: &picture(), bytes: b"<svg/>".to_vec() }.send(issue_key, github_uploads).await
}

fn header(headers: &HeaderMap, name: &str) -> String {
    headers.get(name).and_then(|value| value.to_str().ok()).unwrap_or_default().to_string()
}

/// The field, file name, content type and bytes of the one part a multipart body holds.
async fn only_part(mut form: Multipart) -> (String, String, String, Bytes) {
    let field = form.next_field().await.unwrap().unwrap();
    let name = field.name().unwrap().to_string();
    let file_name = field.file_name().unwrap().to_string();
    let content_type = field.content_type().unwrap().to_string();
    (name, file_name, content_type, field.bytes().await.unwrap())
}

#[tokio::test]
async fn github_takes_the_bytes_against_the_repository_id_and_answers_the_asset_address() {
    let router = Router::new()
        .route("/api/v3/repos/acme/app", get(|| async { Json(json!({ "id": 42 })) }))
        .route(
            "/user-attachments/assets",
            post(|Query(query): Query<HashMap<String, String>>, headers: HeaderMap, body: Bytes| async move {
                assert_eq!(query["name"], "login-3f9a1c2b.svg");
                assert_eq!(query["content_type"], "image/svg+xml");
                assert_eq!(query["repository_id"], "42");
                assert_eq!(header(&headers, "authorization"), "Bearer ghp_secret");
                assert_eq!(header(&headers, "content-type"), "application/octet-stream");
                assert_eq!(header(&headers, "accept"), "application/vnd.github+json");
                assert_eq!(&body[..], b"<svg/>");
                (AxumStatus::CREATED, Json(json!({ "url": "https://github.com/user-attachments/assets/abc" })))
            }),
        );
    let site = serve(router).await;

    let reference = upload(&endpoint(ConnectionKind::Github, &site, "acme/app", "ghp_secret"), None, Some(&site)).await.unwrap();

    assert_eq!(reference, "https://github.com/user-attachments/assets/abc");
}

#[tokio::test]
async fn a_github_404_says_the_token_may_not_write_to_the_repository() {
    let router = Router::new().route("/api/v3/repos/acme/app", get(|| async { Json(json!({ "id": 42 })) }));
    let site = serve(router).await;

    let refused = upload(&endpoint(ConnectionKind::Github, &site, "acme/app", "ghp"), None, Some(&site)).await.unwrap_err();

    assert!(matches!(refused, TrackerFailure::Failed { reason } if reason.contains("may not write to the repository")));
}

#[tokio::test]
async fn a_github_enterprise_server_is_refused_before_anything_is_sent() {
    let refused = upload(&endpoint(ConnectionKind::Github, "https://github.example.com", "acme/app", "ghp"), None, None).await.unwrap_err();

    assert!(matches!(refused, TrackerFailure::Failed { reason } if reason.contains("not supported on GitHub Enterprise Server")));
}

#[tokio::test]
async fn gitlab_takes_a_file_field_and_answers_the_project_address() {
    let router = Router::new().route(
        "/api/v4/projects/{project}/uploads",
        post(|Path(project): Path<String>, headers: HeaderMap, form: Multipart| async move {
            assert_eq!(project, "acme/platform");
            assert_eq!(header(&headers, "private-token"), "glpat");
            let (name, file_name, content_type, bytes) = only_part(form).await;
            assert_eq!((name.as_str(), file_name.as_str(), content_type.as_str(), &bytes[..]), ("file", "login-3f9a1c2b.svg", "image/svg+xml", &b"<svg/>"[..]));
            (AxumStatus::CREATED, Json(json!({ "url": "/uploads/abc/login-3f9a1c2b.svg" })))
        }),
    );
    let site = serve(router).await;

    let reference = upload(&endpoint(ConnectionKind::Gitlab, &site, "acme/platform", "glpat"), None, None).await.unwrap();

    assert_eq!(reference, "/uploads/abc/login-3f9a1c2b.svg");
}

fn jira_attachments(api: &str) -> Router {
    Router::new().route(
        &format!("/rest/api/{api}/issue/{{key}}/attachments"),
        post(|Path(key): Path<String>, headers: HeaderMap, form: Multipart| async move {
            assert_eq!(key, "ENG-3");
            assert_eq!(header(&headers, "x-atlassian-token"), "no-check");
            assert_eq!(only_part(form).await.0, "file");
            Json(json!([{ "filename": "login-3f9a1c2b.svg", "content": "https://acme.atlassian.net/rest/api/3/attachment/content/7" }]))
        }),
    )
}

#[tokio::test]
async fn jira_cloud_refers_to_an_attachment_by_its_content_address() {
    let site = serve(jira_attachments("3")).await;

    let reference = upload(&endpoint(ConnectionKind::Jira, &site, "ENG", "ada@example.com:token"), Some("ENG-3"), None).await.unwrap();

    assert_eq!(reference, "https://acme.atlassian.net/rest/api/3/attachment/content/7");
}

#[tokio::test]
async fn jira_data_center_refers_to_an_attachment_by_its_file_name() {
    let site = serve(jira_attachments("2")).await;

    let reference = upload(&endpoint(ConnectionKind::Jira, &site, "ENG", "pat:token"), Some("ENG-3"), None).await.unwrap();

    assert_eq!(reference, "login-3f9a1c2b.svg");
}

#[tokio::test]
async fn a_refused_credential_is_rejected_and_other_refusals_say_why() {
    let router = Router::new()
        .route("/api/v4/projects/{project}/uploads", post(|| async { AxumStatus::UNAUTHORIZED }))
        .route("/rest/api/2/issue/{key}/attachments", post(|| async { (AxumStatus::PAYLOAD_TOO_LARGE, Json::<Value>(json!({ "errorMessages": ["Too large"] }))) }));
    let site = serve(router).await;

    let rejected = upload(&endpoint(ConnectionKind::Gitlab, &site, "acme/platform", "bad"), None, None).await.unwrap_err();
    let refused = upload(&endpoint(ConnectionKind::Jira, &site, "ENG", "pat:token"), Some("ENG-3"), None).await.unwrap_err();

    assert_eq!(rejected, TrackerFailure::Rejected { status: 401 });
    assert_eq!(refused, TrackerFailure::Failed { reason: "it answered HTTP 413: Too large".into() });
}
