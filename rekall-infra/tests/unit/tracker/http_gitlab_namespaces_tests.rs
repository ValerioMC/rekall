use axum::extract::{Path, Query};
use axum::http::{HeaderMap, StatusCode as AxumStatus};
use axum::routing::get;
use axum::{Json, Router};
use rekall_core::connection::Secret;
use rekall_store::ConnectionKind;
use serde_json::json;
use std::collections::HashMap;

use super::*;

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

fn endpoint(site_url: &str, location: &str) -> TrackerEndpoint {
    TrackerEndpoint { kind: ConnectionKind::Gitlab, site_url: site_url.into(), proxy_url: None, location: location.into(), credential: Secret::new("glpat-secret") }
}

fn gitlab() -> Router {
    Router::new()
        .route(
            "/api/v4/projects/{path}",
            get(|Path(path): Path<String>, headers: HeaderMap| async move {
                assert_eq!(headers.get("private-token").unwrap(), "glpat-secret");
                if path == "acme/billing" {
                    return (AxumStatus::OK, Json(json!({ "path_with_namespace": "acme/billing", "name": "Billing", "visibility": "private", "default_branch": "main", "last_activity_at": "2026-11-18T10:00:00Z" })));
                }
                (AxumStatus::NOT_FOUND, Json(json!({ "message": "404 Project Not Found" })))
            }),
        )
        .route(
            "/api/v4/groups/{path}",
            get(|Path(path): Path<String>| async move {
                if path == "acme" {
                    return (AxumStatus::OK, Json(json!({ "full_path": "acme", "name": "Acme", "visibility": "private" })));
                }
                (AxumStatus::NOT_FOUND, Json(json!({ "message": "404 Group Not Found" })))
            }),
        )
        .route("/api/v4/groups/{path}/subgroups", get(|| async { ([("x-total", "2")], Json(json!([{ "full_path": "acme/libs", "name": "Libs" }]))) }))
        .route("/api/v4/groups/{path}/projects", get(|| async { ([("x-total", "9")], Json(json!([{ "path_with_namespace": "acme/billing", "name": "Billing" }]))) }))
}

#[tokio::test]
async fn a_project_path_is_read_with_its_branch_and_last_activity() {
    let site = serve(gitlab()).await;

    let found = HttpGitlabNamespaces::new().lookup(&endpoint(&site, "acme/billing")).await.unwrap().unwrap();

    assert_eq!((found.kind, found.name.as_str(), found.default_branch.as_deref()), (GitlabNamespaceKind::Project, "Billing", Some("main")));
    assert_eq!(found.last_activity_at.as_deref(), Some("2026-11-18T10:00:00Z"));
}

#[tokio::test]
async fn a_group_path_is_read_with_the_totals_gitlab_gives() {
    let site = serve(gitlab()).await;

    let found = HttpGitlabNamespaces::new().lookup(&endpoint(&site, "acme")).await.unwrap().unwrap();

    assert_eq!((found.kind, found.subgroup_count, found.project_count), (GitlabNamespaceKind::Group, Some(2), Some(9)));
}

#[tokio::test]
async fn a_path_gitlab_has_neither_a_project_nor_a_group_at_is_absent() {
    let site = serve(gitlab()).await;

    assert_eq!(HttpGitlabNamespaces::new().lookup(&endpoint(&site, "acme/nope")).await.unwrap(), None);
}

#[tokio::test]
async fn a_rejected_token_is_a_failure_and_not_an_absent_path() {
    let site = serve(Router::new().route("/api/v4/projects/{path}", get(|| async { AxumStatus::UNAUTHORIZED }))).await;

    let failure = HttpGitlabNamespaces::new().lookup(&endpoint(&site, "acme")).await.unwrap_err();

    assert_eq!(failure, TrackerFailure::Rejected { status: 401 });
}

#[tokio::test]
async fn the_children_of_a_group_are_its_subgroups_then_its_projects() {
    let site = serve(gitlab()).await;

    let children = HttpGitlabNamespaces::new().children(&endpoint(&site, "acme"), "", 10).await.unwrap();

    assert_eq!(children.iter().map(|child| (child.kind, child.path.as_str())).collect::<Vec<_>>(), [(GitlabNamespaceKind::Group, "acme/libs"), (GitlabNamespaceKind::Project, "acme/billing")]);
}

#[tokio::test]
async fn a_blank_location_lists_the_top_level_groups_only() {
    let router = Router::new().route(
        "/api/v4/groups",
        get(|Query(query): Query<HashMap<String, String>>| async move {
            assert_eq!(query.get("top_level_only").map(String::as_str), Some("true"));
            Json(json!([{ "full_path": "acme", "name": "Acme" }]))
        }),
    );
    let site = serve(router).await;

    let children = HttpGitlabNamespaces::new().children(&endpoint(&site, ""), "", 10).await.unwrap();

    assert_eq!(children.len(), 1);
}
