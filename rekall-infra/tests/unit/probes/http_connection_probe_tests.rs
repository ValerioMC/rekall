use axum::http::{HeaderMap, StatusCode as AxumStatus};
use axum::routing::{get, post};
use axum::{Json, Router};

use super::*;

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

fn gitlab_server() -> Router {
    Router::new().route(
        "/api/v4/user",
        get(|headers: HeaderMap| async move {
            match headers.get("PRIVATE-TOKEN").and_then(|value| value.to_str().ok()) {
                Some("glpat-good") => Ok(Json(json!({ "username": "deployer" }))),
                _ => Err(AxumStatus::UNAUTHORIZED),
            }
        }),
    )
}

fn bearer(headers: &HeaderMap) -> Option<String> {
    headers.get("authorization").and_then(|value| value.to_str().ok()).map(str::to_string)
}

/// A cluster too old for `SelfSubjectReview`: it answers 404 there and 200 on `/api` for the right token.
fn old_cluster() -> Router {
    Router::new()
        .route("/version", get(|| async { Json(json!({ "gitVersion": "v1.26.1" })) }))
        .route(SELF_SUBJECT_REVIEW, post(|| async { AxumStatus::NOT_FOUND }))
        .route(
            "/api",
            get(|headers: HeaderMap| async move { if bearer(&headers).as_deref() == Some("Bearer sa-good") { AxumStatus::OK } else { AxumStatus::FORBIDDEN } }),
        )
}

fn cluster() -> Router {
    Router::new()
        .route("/version", get(|| async { Json(json!({ "gitVersion": "v1.30.2" })) }))
        .route(SELF_SUBJECT_REVIEW, post(|| async { Json(json!({ "status": { "userInfo": { "username": "system:serviceaccount:apps:deployer" } } })) }))
}

fn access(server: String, token: &str) -> KubeAccess {
    KubeAccess { context: "prod".into(), server, certificate_authority: None, tls_server_name: None, proxy_url: None, namespace: None, credential: KubeCredential::Token(Secret::new(token)) }
}

#[tokio::test]
async fn a_good_gitlab_token_is_accepted_and_names_its_user() {
    let base = serve(gitlab_server()).await;
    let outcome = HttpConnectionProbe::new().gitlab(&base, &Secret::new("glpat-good")).await;
    assert_eq!(outcome, ProbeOutcome::Accepted { identity: Some("deployer".into()), version: None });
}

#[tokio::test]
async fn a_bad_gitlab_token_is_rejected() {
    let base = serve(gitlab_server()).await;
    let outcome = HttpConnectionProbe::new().gitlab(&base, &Secret::new("glpat-bad")).await;
    assert_eq!(outcome, ProbeOutcome::Rejected { status: 401 });
}

#[tokio::test]
async fn a_cluster_names_the_identity_and_version_of_the_credential() {
    let server = serve(cluster()).await;
    let outcome = HttpConnectionProbe::new().kubernetes(&access(server, "sa-good")).await;
    assert_eq!(outcome, ProbeOutcome::Accepted { identity: Some("system:serviceaccount:apps:deployer".into()), version: Some("v1.30.2".into()) });
}

#[tokio::test]
async fn an_older_cluster_still_proves_the_token_through_its_api_root() {
    let server = serve(old_cluster()).await;
    let probe = HttpConnectionProbe::new();
    assert_eq!(probe.kubernetes(&access(server.clone(), "sa-good")).await, ProbeOutcome::Accepted { identity: None, version: Some("v1.26.1".into()) });
    assert_eq!(probe.kubernetes(&access(server, "sa-bad")).await, ProbeOutcome::Rejected { status: 403 });
}

/// The fake cluster answers the absolute-form requests a proxy receives, so it stands in for the proxy:
/// the server's own host does not resolve, and only a request sent through the proxy gets an answer.
#[tokio::test]
async fn a_cluster_behind_a_proxy_is_asked_through_it() {
    let proxy = serve(cluster()).await;
    let mut behind = access("http://cluster.invalid".into(), "sa-good");
    behind.proxy_url = Some(proxy);
    let outcome = HttpConnectionProbe::new().kubernetes(&behind).await;
    assert_eq!(outcome, ProbeOutcome::Accepted { identity: Some("system:serviceaccount:apps:deployer".into()), version: Some("v1.30.2".into()) });
}

#[tokio::test]
async fn a_server_that_is_not_there_is_unreachable_and_says_why() {
    let outcome = HttpConnectionProbe::new().gitlab("http://127.0.0.1:9", &Secret::new("glpat-good")).await;
    let ProbeOutcome::Unreachable { reason } = outcome else { panic!("an unreachable outcome was expected") };
    assert!(!reason.contains("glpat-good"));
    assert!(reason.to_lowercase().contains("refused"), "the cause is missing from: {reason}");
}

#[tokio::test]
async fn stored_certificate_data_that_is_not_base64_is_said_plainly() {
    let mut broken = access("https://k8s.example.com".into(), "sa-good");
    broken.certificate_authority = Some("not base64!".into());
    let outcome = HttpConnectionProbe::new().kubernetes(&broken).await;
    assert_eq!(outcome, ProbeOutcome::Unreachable { reason: "the stored certificate authority is not base64".into() });
}

fn jira_site() -> Router {
    Router::new().route(
        JIRA_CLOUD_MYSELF,
        get(|headers: HeaderMap| async move {
            let expected = format!("Basic {}", STANDARD.encode("ada@example.com:atl-good"));
            if bearer(&headers) == Some(expected) { Ok(Json(json!({ "displayName": "Ada Lovelace" }))) } else { Err(AxumStatus::UNAUTHORIZED) }
        }),
    )
}

fn direct(url: String) -> JiraSite {
    JiraSite { url, proxy_url: None }
}

#[tokio::test]
async fn a_good_jira_credential_is_accepted_and_names_its_account() {
    let site = direct(serve(jira_site()).await);
    let credential = JiraCredential::cloud("ada@example.com", Secret::new("atl-good")).unwrap();
    let outcome = HttpConnectionProbe::new().jira(&site, &credential).await;
    assert_eq!(outcome, ProbeOutcome::Accepted { identity: Some("Ada Lovelace".into()), version: None });
}

#[tokio::test]
async fn a_bad_jira_credential_is_rejected() {
    let site = direct(serve(jira_site()).await);
    let credential = JiraCredential::cloud("ada@example.com", Secret::new("atl-bad")).unwrap();
    assert_eq!(HttpConnectionProbe::new().jira(&site, &credential).await, ProbeOutcome::Rejected { status: 401 });
}

#[tokio::test]
async fn a_jira_site_behind_a_proxy_is_asked_through_it() {
    let site = JiraSite { url: "http://jira.invalid".into(), proxy_url: Some(serve(jira_site()).await) };
    let credential = JiraCredential::cloud("ada@example.com", Secret::new("atl-good")).unwrap();
    let outcome = HttpConnectionProbe::new().jira(&site, &credential).await;
    assert_eq!(outcome, ProbeOutcome::Accepted { identity: Some("Ada Lovelace".into()), version: None });
}

/// A Data Center site knows only v2 and a personal access token, and names its user by username.
#[tokio::test]
async fn a_data_center_personal_access_token_is_checked_through_v2_and_names_its_username() {
    let site = direct(
        serve(Router::new().route(
            JIRA_DATA_CENTER_MYSELF,
            get(|headers: HeaderMap| async move {
                if bearer(&headers) == Some("Bearer NjQ0Mj".to_string()) { Ok(Json(json!({ "name": "vcasale" }))) } else { Err(AxumStatus::UNAUTHORIZED) }
            }),
        ))
        .await,
    );
    let credential = JiraCredential::data_center(Secret::new("NjQ0Mj")).unwrap();
    let outcome = HttpConnectionProbe::new().jira(&site, &credential).await;
    assert_eq!(outcome, ProbeOutcome::Accepted { identity: Some("vcasale".into()), version: None });
}
