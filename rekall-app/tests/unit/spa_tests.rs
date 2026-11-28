use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use super::*;

const INDEX: &str = "<!doctype html><title>Rekall</title>";

async fn status_of(path: &str) -> StatusCode {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join("index.html"), INDEX).unwrap();
    let router = routes(Assets::new(Some(folder.path().to_path_buf())));
    let request = Request::builder().uri(path).body(Body::empty()).unwrap();
    router.oneshot(request).await.unwrap().status()
}

#[tokio::test]
async fn every_route_of_the_console_answers_with_the_application_on_a_refresh() {
    let routes = [
        "/",
        "/work",
        "/work/board",
        "/task/7d1c2b9e-0000-4000-8000-000000000001",
        "/task/7d1c2b9e-0000-4000-8000-000000000001/description",
        "/task/7d1c2b9e-0000-4000-8000-000000000001/notes/9c",
        "/review",
        "/review/RK-12",
        "/console",
        "/projects",
        "/projects/7d1c/tasks",
        "/companies",
        "/companies/7d1c",
        "/connections",
        "/connections/7d1c",
        "/report",
        "/provider-report",
        "/diagrams",
        "/security",
        "/claude",
        "/settings",
        "/settings/appearance/width",
        "/calendar",
    ];
    for path in routes {
        assert_eq!(status_of(path).await, StatusCode::OK, "{path}");
    }
}

#[tokio::test]
async fn a_path_that_is_neither_a_route_nor_a_file_is_a_404() {
    assert_eq!(status_of("/no-such-page").await, StatusCode::NOT_FOUND);
}
