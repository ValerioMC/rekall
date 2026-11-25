//! A project's blueprint, pages and components written as one PDF or Word file, kept until deleted,
//! listed with their company and project, and cleared by company, project or failure.

mod support;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::{ImageFormat, Rgba, RgbaImage};
use serde_json::{json, Value};
use std::io::Cursor;
use support::{app, id};

const FLOW: &str = "graph TD\n  A --> B";

struct Vega {
    company_id: String,
    project_id: String,
    page_id: String,
    component_id: String,
}

fn png() -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    RgbaImage::from_pixel(40, 20, Rgba([240, 240, 240, 255])).write_to(&mut out, ImageFormat::Png).unwrap();
    out.into_inner()
}

async fn vega(app: &support::App, company_name: &str, label: &str) -> Vega {
    let company_id = app.a_company(company_name).await;
    let project_id = app.a_project(&company_id, label, "ACTIVE").await;
    let written = app
        .put(&format!("/api/projects/{project_id}"), json!({ "label": label, "title": label, "status": "ACTIVE", "companyId": company_id, "blueprintMarkdown": "The blueprint." }))
        .await;
    assert_eq!(written.status, 200, "{}", written.text);
    let page_id = id(&app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Release Plan", "bodyMarkdown": format!("Ship it.\n\n```mermaid\n{FLOW}\n```") })).await);
    let component_id = id(&app.post(&format!("/api/projects/{project_id}/components"), json!({ "name": "API", "folder": "/work/api", "summary": "The REST surface.", "description": "Serves the console." })).await);
    Vega { company_id, project_id, page_id, component_id }
}

fn everything(vega: &Vega, format: &str) -> Value {
    json!({
        "format": format,
        "blueprint": true,
        "sectionIds": [vega.page_id],
        "componentIds": [vega.component_id],
        "diagrams": [{ "source": FLOW, "png": STANDARD.encode(png()) }]
    })
}

async fn export(app: &support::App, vega: &Vega, body: Value) -> support::Resp {
    app.post(&format!("/api/projects/{}/exports", vega.project_id), body).await
}

async fn file(app: &support::App, export_id: &str) -> reqwest::Response {
    app.client.get(app.url(&format!("/api/exports/{export_id}/file"))).send().await.unwrap()
}

async fn listed(app: &support::App) -> Vec<Value> {
    app.get("/api/exports").await.list()
}

#[tokio::test]
async fn each_format_is_written_listed_with_its_project_and_downloaded_as_an_attachment() {
    let app = app().await;
    let vega = vega(&app, "Acme", "vega").await;

    let pdf = export(&app, &vega, everything(&vega, "pdf")).await;
    let docx = export(&app, &vega, everything(&vega, "docx")).await;

    assert_eq!((pdf.status, pdf.str("status"), pdf.str("format"), pdf.str("summary")), (201, "DONE".into(), "PDF".into(), "Blueprint, 1 page, 1 component".into()), "{}", pdf.text);
    assert_eq!((docx.status, docx.str("format"), docx.get("diagramsMissing").as_i64()), (201, "DOCX".into(), Some(0)));
    let pdf_file = file(&app, &pdf.str("id")).await;
    assert_eq!(pdf_file.status(), 200);
    assert_eq!(pdf_file.headers()["content-type"], "application/pdf");
    assert_eq!(pdf_file.headers()["content-disposition"], "attachment; filename=\"vega-export.pdf\"");
    assert!(pdf_file.bytes().await.unwrap().starts_with(b"%PDF-"));
    let docx_file = file(&app, &docx.str("id")).await;
    assert_eq!(docx_file.headers()["content-type"], "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
    assert_eq!(docx_file.headers()["content-disposition"], "attachment; filename=\"vega-export.docx\"");
    assert!(docx_file.bytes().await.unwrap().starts_with(b"PK"));
}

#[tokio::test]
async fn the_list_is_newest_first_with_the_company_and_project_a_page_groups_by() {
    let app = app().await;
    let vega = vega(&app, "Acme", "vega").await;
    let first = export(&app, &vega, json!({ "format": "pdf", "blueprint": true })).await;
    let second = export(&app, &vega, json!({ "format": "docx", "componentIds": [vega.component_id] })).await;

    let exports = listed(&app).await;

    assert_eq!(exports.iter().map(|entry| entry["id"].as_str().unwrap()).collect::<Vec<_>>(), [second.str("id"), first.str("id")]);
    assert_eq!(exports[0]["summary"], "API");
    assert_eq!(exports[0]["fileName"], "vega-api.docx");
    assert_eq!((exports[0]["companyName"].as_str(), exports[0]["projectLabel"].as_str()), (Some("Acme"), Some("vega")));
    assert_eq!((exports[0]["companyId"].as_str(), exports[0]["projectId"].as_str()), (Some(vega.company_id.as_str()), Some(vega.project_id.as_str())));
}

#[tokio::test]
async fn a_diagram_with_no_picture_is_counted_and_does_not_stop_the_export() {
    let app = app().await;
    let vega = vega(&app, "Acme", "vega").await;

    let written = export(&app, &vega, json!({ "format": "pdf", "sectionIds": [vega.page_id] })).await;

    assert_eq!((written.status, written.get("diagramsMissing").as_i64()), (201, Some(1)));
}

#[tokio::test]
async fn deleting_an_export_removes_it_and_its_file_answers_404() {
    let app = app().await;
    let vega = vega(&app, "Acme", "vega").await;
    let written = export(&app, &vega, json!({ "format": "pdf", "blueprint": true })).await;

    assert_eq!(app.delete(&format!("/api/exports/{}", written.str("id"))).await.status, 204);

    assert!(listed(&app).await.is_empty());
    assert_eq!(file(&app, &written.str("id")).await.status(), 404);
    assert_eq!(app.count("SELECT COUNT(*) FROM project_export_file", vec![]).await, 0);
    assert_eq!(app.delete(&format!("/api/exports/{}", written.str("id"))).await.status, 404);
}

#[tokio::test]
async fn an_empty_selection_and_a_bad_request_are_refused_and_leave_no_export() {
    let app = app().await;
    let vega = vega(&app, "Acme", "vega").await;

    let nothing = export(&app, &vega, json!({ "format": "pdf" })).await;
    let no_format = export(&app, &vega, json!({ "blueprint": true })).await;
    let unknown_format = export(&app, &vega, json!({ "format": "rtf", "blueprint": true })).await;
    let bad_picture = export(&app, &vega, json!({ "format": "pdf", "blueprint": true, "diagrams": [{ "source": FLOW, "png": "***" }] })).await;

    assert_eq!(nothing.status, 400, "{}", nothing.text);
    assert_eq!((no_format.status, unknown_format.status, bad_picture.status), (400, 400, 400));
    assert!(listed(&app).await.is_empty());
}

#[tokio::test]
async fn a_page_of_another_project_is_refused() {
    let app = app().await;
    let vega = vega(&app, "Acme", "vega").await;
    let atlas = self::vega(&app, "Globex", "atlas").await;

    let refused = export(&app, &vega, json!({ "format": "pdf", "sectionIds": [atlas.page_id] })).await;
    let refused_component = export(&app, &vega, json!({ "format": "pdf", "componentIds": [atlas.component_id] })).await;

    assert_eq!((refused.status, refused_component.status), (404, 404));
    assert!(listed(&app).await.is_empty());
}

#[tokio::test]
async fn a_picture_the_writer_cannot_read_is_a_500_that_names_the_kind_and_is_listed_as_failed() {
    let app = app().await;
    let vega = vega(&app, "Acme", "vega").await;
    let body = json!({ "format": "pdf", "sectionIds": [vega.page_id], "diagrams": [{ "source": FLOW, "png": STANDARD.encode(b"not a png") }] });

    let failed = export(&app, &vega, body).await;

    assert_eq!(failed.status, 500, "{}", failed.text);
    assert!(failed.text.contains("Render error"), "{}", failed.text);
    let exports = listed(&app).await;
    assert_eq!((exports[0]["status"].as_str(), exports[0]["failureKind"].as_str()), (Some("FAILED"), Some("RENDER")));
    assert!(exports[0]["failureMessage"].as_str().unwrap().starts_with("The PDF could not be written"));
    assert_eq!(file(&app, exports[0]["id"].as_str().unwrap()).await.status(), 404);
}

#[tokio::test]
async fn exports_are_cleared_by_company_project_or_failure_and_a_clear_with_no_criterion_is_refused() {
    let app = app().await;
    let vega = vega(&app, "Acme", "vega").await;
    let atlas = self::vega(&app, "Globex", "atlas").await;
    export(&app, &vega, json!({ "format": "pdf", "blueprint": true })).await;
    export(&app, &atlas, json!({ "format": "pdf", "blueprint": true })).await;
    export(&app, &atlas, json!({ "format": "pdf", "sectionIds": [atlas.page_id], "diagrams": [{ "source": FLOW, "png": STANDARD.encode(b"not a png") }] })).await;

    assert_eq!(app.delete("/api/exports").await.status, 400);
    assert_eq!(app.delete("/api/exports?status=failed").await.get("deleted").as_u64(), Some(1));
    assert_eq!(app.delete(&format!("/api/exports?projectId={}", atlas.project_id)).await.get("deleted").as_u64(), Some(1));
    assert_eq!(listed(&app).await.len(), 1);
    assert_eq!(app.delete(&format!("/api/exports?companyId={}", vega.company_id)).await.get("deleted").as_u64(), Some(1));
    assert!(listed(&app).await.is_empty());
    assert_eq!(app.delete("/api/exports?status=sleeping").await.status, 400);
}

#[tokio::test]
async fn deleting_a_project_removes_its_exports() {
    let app = app().await;
    let vega = vega(&app, "Acme", "vega").await;
    export(&app, &vega, json!({ "format": "docx", "blueprint": true })).await;

    app.delete(&format!("/api/projects/{}", vega.project_id)).await;

    assert!(listed(&app).await.is_empty());
    assert_eq!(app.count("SELECT COUNT(*) FROM project_export_file", vec![]).await, 0);
}
