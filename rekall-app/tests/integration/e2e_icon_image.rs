//! A project's uploaded icon: stored as one normalised square, served at the exact size a spot on
//! screen asks for, and announced on the project by a version that changes with every upload.

mod support;

use image::{ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;
use support::app;

fn jpeg(width: u32, height: u32) -> Vec<u8> {
    let rgb = image::DynamicImage::ImageRgba8(RgbaImage::from_pixel(width, height, Rgba([30, 120, 200, 255]))).to_rgb8();
    let mut out = Cursor::new(Vec::new());
    rgb.write_to(&mut out, ImageFormat::Jpeg).unwrap();
    out.into_inner()
}

async fn upload(app: &support::App, project_id: &str, bytes: Vec<u8>) -> reqwest::Response {
    app.client.put(app.url(&format!("/api/projects/{project_id}/icon-image"))).body(bytes).send().await.unwrap()
}

async fn rendered_side(app: &support::App, project_id: &str, size: u32) -> (u32, String) {
    let response = app.client.get(app.url(&format!("/api/projects/{project_id}/icon-image?size={size}"))).send().await.unwrap();
    let content_type = response.headers()["content-type"].to_str().unwrap().to_string();
    let image = image::load_from_memory(&response.bytes().await.unwrap()).unwrap();
    (image.width(), content_type)
}

#[tokio::test]
async fn an_upload_of_any_shape_is_served_as_a_png_of_the_size_asked() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    assert!(app.get(&format!("/api/projects/{project_id}")).await.get("iconImageVersion").is_null(), "no picture until one is uploaded");

    let stored = upload(&app, &project_id, jpeg(800, 400)).await;
    assert_eq!(stored.status(), 200);
    let project: serde_json::Value = stored.json().await.unwrap();
    assert!(project["iconImageVersion"].is_string(), "{project}");

    assert_eq!(rendered_side(&app, &project_id, 28).await, (28, "image/png".to_string()));
    assert_eq!(rendered_side(&app, &project_id, 72).await.0, 72);
    let listed = app.get("/api/projects").await.list();
    assert_eq!(listed[0]["iconImageVersion"], project["iconImageVersion"], "the list carries the same version");
}

#[tokio::test]
async fn a_new_upload_changes_the_version_and_removing_it_returns_to_the_icon_key() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let first: serde_json::Value = upload(&app, &project_id, jpeg(64, 64)).await.json().await.unwrap();
    let second: serde_json::Value = upload(&app, &project_id, jpeg(64, 64)).await.json().await.unwrap();
    assert_ne!(first["iconImageVersion"], second["iconImageVersion"]);

    assert_eq!(app.delete(&format!("/api/projects/{project_id}/icon-image")).await.status, 204);

    assert!(app.get(&format!("/api/projects/{project_id}")).await.get("iconImageVersion").is_null());
    let gone = app.client.get(app.url(&format!("/api/projects/{project_id}/icon-image"))).send().await.unwrap();
    assert_eq!(gone.status(), 404);
}

#[tokio::test]
async fn a_file_that_is_not_an_image_and_an_unknown_project_are_refused() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;

    assert_eq!(upload(&app, &project_id, b"not a picture".to_vec()).await.status(), 400);
    assert_eq!(upload(&app, "00000000-0000-4000-8000-000000000000", jpeg(32, 32)).await.status(), 404);
    assert!(app.get(&format!("/api/projects/{project_id}")).await.get("iconImageVersion").is_null());
}

#[tokio::test]
async fn deleting_a_project_deletes_its_picture() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    upload(&app, &project_id, jpeg(32, 32)).await;

    app.delete(&format!("/api/projects/{project_id}")).await;

    assert_eq!(app.count("SELECT COUNT(*) FROM project_icon_image", vec![]).await, 0);
}
