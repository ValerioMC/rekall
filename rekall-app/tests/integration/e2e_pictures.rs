//! Keeping pictures beside the database and serving them back, over HTTP.

mod support;

use serde_json::json;
use support::{app, App};

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n pixels";
const SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="1" height="1"/></svg>"#;
const SVG_POLICY: &str = "default-src 'none'; style-src 'unsafe-inline'; img-src data:; sandbox";

async fn a_task(app: &App) -> String {
    let company = app.post("/api/companies", json!({ "name": "Acme" })).await;
    let project = app.post("/api/projects", json!({ "label": "web", "title": "Web", "taskKey": "WEB", "companyId": company.str("id") })).await;
    let task = app.post("/api/tasks", json!({ "title": "Login", "projectId": project.str("id") })).await;
    task.str("id")
}

async fn import(app: &App, task: &str, name: &str, bytes: &[u8]) -> String {
    let source = app.folder.path().join(name);
    std::fs::write(&source, bytes).unwrap();
    let imported = app.post("/api/images", json!({ "sourcePath": source, "owner": { "kind": "TASK", "id": task } })).await;
    assert_eq!(imported.status, 201, "{}", imported.text);
    imported.str("url")
}

async fn served(app: &App, url: &str) -> reqwest::Response {
    app.client.get(app.url(url)).send().await.unwrap()
}

fn header<'a>(response: &'a reqwest::Response, name: &str) -> Option<&'a str> {
    response.headers().get(name).map(|value| value.to_str().unwrap())
}

#[tokio::test]
async fn an_imported_png_and_svg_are_copied_beside_the_database_and_served_with_their_headers() {
    let app = app().await;
    let task = a_task(&app).await;

    let png_url = import(&app, &task, "Login Flow.png", PNG).await;
    let svg_url = import(&app, &task, "Diagram.svg", SVG).await;

    assert!(png_url.starts_with("/api/images/acme/web/web-1/login-flow-"), "{png_url}");
    let copied = app.folder.path().join("db/images").join(png_url.strip_prefix("/api/images/").unwrap());
    assert_eq!(std::fs::read(copied).unwrap(), PNG);

    let png = served(&app, &png_url).await;
    assert_eq!(png.status(), 200);
    assert_eq!(header(&png, "content-type"), Some("image/png"));
    assert_eq!(header(&png, "cache-control"), Some("public, max-age=31536000, immutable"));
    assert_eq!(header(&png, "x-content-type-options"), Some("nosniff"));
    assert_eq!(header(&png, "cross-origin-resource-policy"), Some("same-origin"));
    assert_eq!(header(&png, "content-security-policy"), None);
    assert_eq!(png.bytes().await.unwrap().as_ref(), PNG);

    let svg = served(&app, &svg_url).await;
    assert_eq!(header(&svg, "content-type"), Some("image/svg+xml"));
    assert_eq!(header(&svg, "content-security-policy"), Some(SVG_POLICY));
    assert_eq!(svg.bytes().await.unwrap().as_ref(), SVG);

    assert_eq!(app.get("/api/images/acme/web/..%2F..%2Frekall.db").await.status, 404);
    assert_eq!(app.get("/api/images/acme/web/web-1/..%2F..%2F..%2F..%2Frekall.png").await.status, 404);

    app.stop().await;
}

async fn thumbnail_of(app: &App, name: &str, bytes: &[u8]) -> reqwest::Response {
    let source = app.folder.path().join(name);
    std::fs::write(&source, bytes).unwrap();
    let url = app.url("/api/image-thumbnail");
    app.client.get(url).query(&[("path", source.to_str().unwrap())]).send().await.unwrap()
}

#[tokio::test]
async fn a_file_being_browsed_is_previewed_under_the_headers_of_a_kept_picture() {
    let app = app().await;

    let svg = thumbnail_of(&app, "Logo & mark.svg", SVG).await;
    assert_eq!(svg.status(), 200);
    assert_eq!(header(&svg, "content-type"), Some("image/svg+xml"));
    assert_eq!(header(&svg, "content-security-policy"), Some(SVG_POLICY));
    assert_eq!(header(&svg, "cache-control"), Some("private, no-cache"));
    assert_eq!(header(&svg, "x-content-type-options"), Some("nosniff"));
    assert_eq!(header(&svg, "cross-origin-resource-policy"), Some("same-origin"));
    assert_eq!(svg.bytes().await.unwrap().as_ref(), SVG);

    assert_eq!(thumbnail_of(&app, "notes.png", b"only text").await.status(), 400);
    assert_eq!(app.get("/api/image-thumbnail").await.status, 400);
    assert_eq!(app.get("/api/image-thumbnail?path=relative.png").await.status, 400);
    assert!(!app.folder.path().join("db/images").exists());

    app.stop().await;
}

