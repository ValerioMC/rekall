use axum::http::{HeaderMap, StatusCode as AxumStatus};
use axum::routing::post;
use axum::{Json, Router};
use rekall_service::connection::Secret;

use super::*;

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

/// Answers a search with the good credential with one issue, echoing the JQL it was sent as its summary.
fn jira_site() -> Router {
    Router::new().route(
        CLOUD_SEARCH,
        post(|headers: HeaderMap, Json(body): Json<serde_json::Value>| async move {
            let signed = headers.get("authorization").and_then(|value| value.to_str().ok()) == Some("Basic YWRhQGV4YW1wbGUuY29tOmF0bC1nb29k");
            if !signed {
                return Err(AxumStatus::UNAUTHORIZED);
            }
            if body["jql"] == "broken" {
                return Ok((AxumStatus::BAD_REQUEST, Json(json!({ "errorMessages": ["The value 'NOPE' does not exist for the field 'project'."] }))));
            }
            Ok((
                AxumStatus::OK,
                Json(json!({ "issues": [{
                    "key": "ENG-1",
                    "fields": {
                        "summary": body["jql"],
                        "status": { "name": "To Do" },
                        "issuetype": { "name": "Story" },
                        "description": { "type": "doc", "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "Steps" }] }] }
                    }
                }]})),
            ))
        }),
    )
}

/// A Data Center site: v2's search, a personal access token as Bearer, the description as text.
fn data_center_site() -> Router {
    Router::new().route(
        DATA_CENTER_SEARCH,
        post(|headers: HeaderMap| async move {
            if headers.get("authorization").and_then(|value| value.to_str().ok()) != Some("Bearer NjQ0Mj") {
                return Err(AxumStatus::UNAUTHORIZED);
            }
            Ok(Json(json!({ "issues": [{ "key": "OPS-7", "fields": { "summary": "Rotate keys", "description": " h2. Steps \n" } }] })))
        }),
    )
}

fn direct(url: String) -> JiraSite {
    JiraSite { url, proxy_url: None }
}

fn credential(token: &str) -> JiraCredential {
    JiraCredential::cloud("ada@example.com", Secret::new(token)).unwrap()
}

#[tokio::test]
async fn a_data_center_site_is_searched_through_v2_with_its_description_as_text() {
    let site = direct(serve(data_center_site()).await);

    let issues = HttpJiraIssues::new().search(&site, &JiraCredential::data_center(Secret::new("NjQ0Mj")).unwrap(), "project = \"OPS\"", 50).await.unwrap();

    assert_eq!(
        issues,
        [JiraIssue { key: "OPS-7".into(), summary: "Rotate keys".into(), status: None, issue_type: None, description: Some("h2. Steps".into()) }]
    );
}

#[tokio::test]
async fn a_search_reads_each_issue_with_its_description_as_markdown() {
    let site = direct(serve(jira_site()).await);

    let issues = HttpJiraIssues::new().search(&site, &credential("atl-good"), "project = \"ENG\"", 50).await.unwrap();

    assert_eq!(
        issues,
        [JiraIssue {
            key: "ENG-1".into(),
            summary: "project = \"ENG\"".into(),
            status: Some("To Do".into()),
            issue_type: Some("Story".into()),
            description: Some("Steps".into()),
        }]
    );
}

#[tokio::test]
async fn a_rejected_credential_is_told_apart_from_a_refused_search() {
    let site = direct(serve(jira_site()).await);

    let rejected = HttpJiraIssues::new().search(&site, &credential("atl-bad"), "project = \"ENG\"", 50).await.unwrap_err();
    let refused = HttpJiraIssues::new().search(&site, &credential("atl-good"), "broken", 50).await.unwrap_err();

    assert_eq!(rejected, JiraFailure::Rejected { status: 401 });
    assert_eq!(refused, JiraFailure::Failed { reason: "it answered HTTP 400: The value 'NOPE' does not exist for the field 'project'.".into() });
}

/// The fake site answers the absolute-form requests a proxy receives, so it stands in for the proxy:
/// the site's own host does not resolve, and only a search sent through the proxy gets an answer.
#[tokio::test]
async fn a_site_behind_a_proxy_is_searched_through_it() {
    let site = JiraSite { url: "http://jira.invalid".into(), proxy_url: Some(serve(jira_site()).await) };

    let issues = HttpJiraIssues::new().search(&site, &credential("atl-good"), "project = \"ENG\"", 50).await.unwrap();

    assert_eq!(issues[0].key, "ENG-1");
}

#[tokio::test]
async fn a_proxy_reqwest_cannot_use_is_said_plainly() {
    let site = JiraSite { url: "https://example.atlassian.net".into(), proxy_url: Some("not a proxy".into()) };

    let failure = HttpJiraIssues::new().search(&site, &credential("atl-good"), "project = \"ENG\"", 50).await.unwrap_err();

    assert_eq!(failure, JiraFailure::Failed { reason: "the connection's proxy is not a usable proxy address".into() });
}
