use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode as AxumStatus};
use axum::routing::get;
use axum::{Json, Router};
use rekall_core::connection::Secret;
use serde_json::json;
use std::collections::HashMap;

use super::*;

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

fn signed(headers: &HeaderMap) -> bool {
    headers.get("authorization").and_then(|value| value.to_str().ok()) == Some("Bearer NjQ0Mj")
}

fn data_center_site() -> Router {
    Router::new()
        .route(
            "/rest/api/space/ENG",
            get(|headers: HeaderMap| async move {
                if !signed(&headers) {
                    return Err(AxumStatus::UNAUTHORIZED);
                }
                Ok(Json(json!({ "key": "ENG", "name": "Engineering", "description": { "plain": { "value": " How we build \n" } }, "homepage": { "id": "10" } })))
            }),
        )
        .route(
            "/rest/api/space",
            get(|headers: HeaderMap| async move {
                if !signed(&headers) {
                    return Err(AxumStatus::UNAUTHORIZED);
                }
                Ok(Json(json!({ "results": [{ "key": "ENG", "name": "Engineering" }, { "key": "OPS", "name": "Operations" }] })))
            }),
        )
        .route(
            "/rest/api/content",
            get(|headers: HeaderMap, Query(query): Query<HashMap<String, String>>| async move {
                if !signed(&headers) {
                    return Err(AxumStatus::UNAUTHORIZED);
                }
                let first_batch = query["start"] == "0";
                let page = if first_batch {
                    json!({ "id": "10", "title": "Home", "body": { "storage": { "value": "<p>Welcome</p>" } }, "metadata": { "labels": { "results": [{ "name": "overview" }] } }, "_links": { "webui": "/display/ENG/Home" } })
                } else {
                    json!({ "id": "11", "title": "Deploy", "body": { "storage": { "value": "<h2>Steps</h2>" } }, "_links": { "webui": "/display/ENG/Deploy" } })
                };
                let next = if first_batch { json!("/rest/api/content?start=1") } else { serde_json::Value::Null };
                Ok(Json(json!({ "results": [page], "_links": { "base": "https://wiki.example.com", "next": next } })))
            }),
        )
}

fn direct(url: String) -> AtlassianSite {
    AtlassianSite { url, proxy_url: None }
}

fn token() -> AtlassianCredential {
    AtlassianCredential::data_center(Secret::new("NjQ0Mj")).unwrap()
}

#[tokio::test]
async fn a_space_is_read_with_its_plain_description_and_home_page() {
    let site = direct(serve(data_center_site()).await);

    let space = HttpConfluenceSpaces::new().space(&site, &token(), "ENG").await.unwrap();

    assert_eq!(space, Some(ConfluenceSpace { key: "ENG".into(), name: "Engineering".into(), description: Some("How we build".into()), home_page_id: Some("10".into()) }));
}

#[tokio::test]
async fn pages_are_read_batch_by_batch_with_markdown_bodies_labels_and_addresses() {
    let site = direct(serve(data_center_site()).await);

    let pages = HttpConfluenceSpaces::new().pages(&site, &token(), "ENG", 10).await.unwrap();

    assert_eq!(
        pages,
        [
            ConfluencePage { id: "10".into(), title: "Home".into(), labels: vec!["overview".into()], markdown: "Welcome".into(), url: "https://wiki.example.com/display/ENG/Home".into() },
            ConfluencePage { id: "11".into(), title: "Deploy".into(), labels: vec![], markdown: "## Steps".into(), url: "https://wiki.example.com/display/ENG/Deploy".into() },
        ]
    );
}

#[tokio::test]
async fn no_more_pages_than_the_limit_are_asked_for() {
    let site = direct(serve(data_center_site()).await);

    let pages = HttpConfluenceSpaces::new().pages(&site, &token(), "ENG", 1).await.unwrap();

    assert_eq!(pages.len(), 1);
}

#[tokio::test]
async fn a_rejected_credential_and_a_missing_space_are_told_apart() {
    let site = direct(serve(data_center_site()).await);
    let wrong = AtlassianCredential::data_center(Secret::new("wrong")).unwrap();

    let rejected = HttpConfluenceSpaces::new().space(&site, &wrong, "ENG").await.unwrap_err();
    let missing = HttpConfluenceSpaces::new().space(&site, &token(), "NOPE").await.unwrap();

    assert_eq!(rejected, AtlassianFailure::Rejected { status: 401 });
    assert_eq!(missing, None);
}

#[tokio::test]
async fn spaces_are_listed_and_narrowed_by_key_or_name_ignoring_case() {
    let site = direct(serve(data_center_site()).await);

    let everything = HttpConfluenceSpaces::new().spaces(&site, &token(), "", 10).await.unwrap();
    let narrowed = HttpConfluenceSpaces::new().spaces(&site, &token(), "oper", 10).await.unwrap();
    let capped = HttpConfluenceSpaces::new().spaces(&site, &token(), "", 1).await.unwrap();

    assert_eq!(everything.iter().map(|space| space.key.as_str()).collect::<Vec<_>>(), ["ENG", "OPS"]);
    assert_eq!(narrowed.iter().map(|space| space.key.as_str()).collect::<Vec<_>>(), ["OPS"]);
    assert_eq!(capped.len(), 1);
}

#[test]
fn a_cloud_site_without_its_wiki_context_gets_it_and_a_data_center_site_does_not() {
    let cloud = direct("https://example.atlassian.net/".into());
    let wiki = direct("https://example.atlassian.net/wiki".into());
    let data_center = direct("https://wiki.example.com/confluence".into());

    assert_eq!(rest_root(&cloud, AtlassianDeployment::Cloud), "https://example.atlassian.net/wiki/rest/api");
    assert_eq!(rest_root(&wiki, AtlassianDeployment::Cloud), "https://example.atlassian.net/wiki/rest/api");
    assert_eq!(rest_root(&data_center, AtlassianDeployment::DataCenter), "https://wiki.example.com/confluence/rest/api");
}
