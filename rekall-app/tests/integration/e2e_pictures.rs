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


async fn uploaded(app: &App, query: &[(&str, &str)], bytes: Vec<u8>) -> (u16, serde_json::Value) {
    let request = app.client.post(app.url("/api/image-uploads")).query(query).header("content-type", "image/png");
    let response = request.body(bytes).send().await.unwrap();
    let status = response.status().as_u16();
    (status, response.json().await.unwrap_or_default())
}

#[tokio::test]
async fn uploaded_bytes_are_kept_as_a_picture_of_their_owner() {
    let app = app().await;
    let task = a_task(&app).await;
    let owner = [("ownerKind", "TASK"), ("ownerId", task.as_str())];

    let (status, pasted) = uploaded(&app, &owner, PNG.to_vec()).await;
    assert_eq!(status, 201, "{pasted}");
    let url = pasted["url"].as_str().unwrap();
    assert!(url.starts_with("/api/images/acme/web/web-1/pasted-image-"), "{url}");
    assert_eq!(served(&app, url).await.bytes().await.unwrap().as_ref(), PNG);

    let named = [owner[0], owner[1], ("name", "Login Flow & co.png")];
    let (_, dropped) = uploaded(&app, &named, PNG.to_vec()).await;
    assert!(dropped["url"].as_str().unwrap().contains("/web-1/login-flow-co-"), "{dropped}");

    app.stop().await;
}

#[tokio::test]
async fn an_upload_refused_for_its_content_size_or_owner_keeps_nothing() {
    let app = app().await;
    let task = a_task(&app).await;
    let owner = [("ownerKind", "TASK"), ("ownerId", task.as_str())];
    let mut at_the_limit = PNG.to_vec();
    at_the_limit.resize(10 * 1024 * 1024, 0);
    let mut over_the_limit = at_the_limit.clone();
    over_the_limit.push(0);

    assert_eq!(uploaded(&app, &owner, b"only text".to_vec()).await.0, 400);
    assert_eq!(uploaded(&app, &owner, Vec::new()).await.0, 400);
    let (status, refusal) = uploaded(&app, &owner, over_the_limit).await;
    assert_eq!((status, refusal["detail"].as_str()), (400, Some("The picture is larger than 10 MB.")));
    assert_eq!(uploaded(&app, &[("ownerKind", "TASK")], PNG.to_vec()).await.0, 400);
    assert_eq!(uploaded(&app, &[("ownerKind", "TASK"), ("ownerId", "nope")], PNG.to_vec()).await.0, 400);
    assert_eq!(uploaded(&app, &[("ownerKind", "FOLDER"), ("ownerId", task.as_str())], PNG.to_vec()).await.0, 400);
    let stranger = [("ownerKind", "PAGE"), ("ownerId", task.as_str())];
    assert_eq!(uploaded(&app, &stranger, PNG.to_vec()).await.0, 404);
    assert!(!app.folder.path().join("db/images").exists());

    assert_eq!(uploaded(&app, &owner, at_the_limit).await.0, 201);

    app.stop().await;
}
