use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use image::{ImageFormat, Rgb, RgbImage};
use rekall_core::picture::{PicturePath, PictureStore, StoredPicture};
use rekall_core::EventBus;
use rekall_kernel::Instant;
use rekall_store::{company, project, project_section, section_context, ExportFailureKind, ProjectStatus};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use super::*;
use crate::export::DiagramImageCommand;

const DIAGRAM_PNG: &[u8] = include_bytes!("../../../fixtures/diagram.png");
const FLOW: &str = "graph TD\n  A --> B";

#[derive(Default)]
struct FakePictures {
    kept: Mutex<HashMap<String, Vec<u8>>>,
}

impl FakePictures {
    fn keep(&self, relative: &str, bytes: Vec<u8>) {
        self.kept.lock().unwrap().insert(relative.to_string(), bytes);
    }
}

#[async_trait]
impl PictureStore for FakePictures {
    fn folder(&self) -> Option<PathBuf> {
        None
    }

    async fn read_source(&self, _source: &Path, _limit_bytes: u64) -> Result<Vec<u8>> {
        unreachable!("an export copies no picture in")
    }

    async fn write(&self, _path: &PicturePath, _bytes: &[u8]) -> Result<()> {
        unreachable!("an export writes no picture")
    }

    async fn read(&self, path: &PicturePath) -> Result<Vec<u8>> {
        self.kept.lock().unwrap().get(path.relative()).cloned().ok_or_else(|| RekallError::not_found_msg("no picture"))
    }

    async fn list(&self) -> Result<Vec<StoredPicture>> {
        unreachable!("an export lists no picture")
    }

    async fn delete(&self, _path: &PicturePath) -> Result<()> {
        unreachable!("an export deletes no picture")
    }
}

struct World {
    service: ProjectExportService,
    company: company::Model,
    vega: project::Model,
    guides: section_context::Model,
    decisions: section_context::Model,
    pages: Vec<project_section::Model>,
    components: Vec<project_component::Model>,
    store: Arc<FakePictures>,
    database: rekall_store::Database,
}

async fn a_project(db: &sea_orm::DatabaseConnection, company_id: Id, label: &str, key: &str, blueprint: Option<&str>, now: Instant) -> project::Model {
    let project = project::Model {
        id: Id::random(), label: label.into(), title: label.to_uppercase(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: blueprint.map(str::to_string), repo_folder: None, auto_commit: false, main_branch: None,
        pull_request_flow: false, task_key: key.into(), company_id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    project
}

async fn a_context(db: &sea_orm::DatabaseConnection, project: &project::Model, title: &str, position: i32, now: Instant) -> section_context::Model {
    let context = section_context::Model { id: Id::random(), project_id: project.id, title: title.into(), position, created_at: now, updated_at: now };
    context.clone().into_active_model().insert(db).await.unwrap();
    context
}

async fn a_page(db: &sea_orm::DatabaseConnection, context: &section_context::Model, title: &str, body: &str, position: i32, now: Instant) -> project_section::Model {
    let page = project_section::Model {
        id: Id::random(), project_id: context.project_id, context_id: context.id, title: title.into(), body_markdown: body.into(),
        position, created_at: now, updated_at: now,
    };
    page.clone().into_active_model().insert(db).await.unwrap();
    page
}

async fn a_component(db: &sea_orm::DatabaseConnection, project: &project::Model, name: &str, position: i32, now: Instant) -> project_component::Model {
    let component = project_component::Model {
        id: Id::random(), project_id: project.id, name: name.into(), folder: format!("/work/{name}"), summary: format!("The {name}"),
        description: format!("About the {name}."), position, created_at: now, updated_at: now,
    };
    component.clone().into_active_model().insert(db).await.unwrap();
    component
}

async fn world() -> World {
    let database = rekall_store::open_temporary().await.unwrap();
    let mut ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::parse("2026-11-25T12:00:00Z").unwrap();
    ctx.clock = Arc::new(move || now);
    let db = &database.conn;
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let vega = a_project(db, company.id, "vega", "VG", Some("The blueprint."), now).await;
    let guides = a_context(db, &vega, "Guides", 0, now).await;
    let decisions = a_context(db, &vega, "Decisions", 1, now).await;
    let pages = vec![
        a_page(db, &decisions, "Why SQLite", "Because.", 0, now).await,
        a_page(db, &guides, "Setup", &format!("Install.\n\n```mermaid\n{FLOW}\n```"), 0, now).await,
        a_page(db, &guides, "Release Plan", "Ship it.", 1, now).await,
    ];
    let components = vec![a_component(db, &vega, "api", 1, now).await, a_component(db, &vega, "ui", 0, now).await];
    let store = Arc::new(FakePictures::default());
    let service = ProjectExportService::new(ctx.clone(), PictureService::new(ctx, store.clone()));
    World { service, company, vega, guides, decisions, pages, components, store, database }
}

fn command(format: ExportFormat) -> ProjectExportCommand {
    ProjectExportCommand { format, blueprint: false, section_ids: vec![], component_ids: vec![], diagrams: vec![] }
}

fn diagram(source: &str, png: &[u8]) -> DiagramImageCommand {
    DiagramImageCommand { source: source.into(), png: png.to_vec() }
}

async fn plan_of(world: &World, command: ProjectExportCommand) -> Result<ExportPlan> {
    plan_for(world, world.vega.id, command).await
}

async fn plan_for(world: &World, project_id: Id, command: ProjectExportCommand) -> Result<ExportPlan> {
    in_read!(&world.service.ctx, |tx| plan_in(&tx, project_id, &command).await)
}

async fn recorded(world: &World) -> Vec<project_export::Model> {
    project_export::Entity::find().all(&world.database.conn).await.unwrap()
}

#[tokio::test]
async fn the_document_holds_the_blueprint_then_the_pages_in_the_project_order_then_the_components_by_position() {
    let world = world().await;
    let plan = plan_of(&world, ProjectExportCommand {
        blueprint: true,
        section_ids: world.pages.iter().map(|page| page.id).collect(),
        component_ids: world.components.iter().map(|component| component.id).collect(),
        ..command(ExportFormat::Pdf)
    })
    .await
    .unwrap();

    let headings: Vec<&str> = plan.document.parts.iter().map(|part| part.heading.as_str()).collect();

    assert_eq!(headings, ["Blueprint", "Setup", "Release Plan", "Why SQLite", "ui", "api"]);
}

#[tokio::test]
async fn a_page_of_another_project_is_refused_and_not_recorded() {
    let world = world().await;
    let now = Instant::now();
    let atlas = a_project(&world.database.conn, world.company.id, "atlas", "AT", None, now).await;
    let elsewhere = a_context(&world.database.conn, &atlas, "Other", 0, now).await;
    let foreign = a_page(&world.database.conn, &elsewhere, "Foreign", "text", 0, now).await;

    let refusal = world.service.export(world.vega.id, ProjectExportCommand { section_ids: vec![foreign.id], ..command(ExportFormat::Pdf) }).await.unwrap_err();

    assert!(matches!(refusal, RekallError::NotFound(_)));
    assert!(recorded(&world).await.is_empty());
}

#[tokio::test]
async fn a_component_that_does_not_exist_is_refused_and_not_recorded() {
    let world = world().await;

    let refusal = world.service.export(world.vega.id, ProjectExportCommand { component_ids: vec![Id::random()], ..command(ExportFormat::Docx) }).await.unwrap_err();

    assert!(matches!(refusal, RekallError::NotFound(_)));
    assert!(recorded(&world).await.is_empty());
}

#[tokio::test]
async fn an_empty_selection_is_refused_with_what_to_tick_and_not_recorded() {
    let world = world().await;

    let refusal = world.service.export(world.vega.id, command(ExportFormat::Pdf)).await.unwrap_err();

    assert!(refusal.is_illegal_argument());
    assert!(refusal.message().contains("Tick"));
    assert!(recorded(&world).await.is_empty());
}

#[tokio::test]
async fn a_blank_blueprint_ticked_alone_is_refused_and_ticked_with_a_page_is_left_out() {
    let world = world().await;
    let now = Instant::now();
    let empty = a_project(&world.database.conn, world.company.id, "empty", "EM", Some("  "), now).await;
    let context = a_context(&world.database.conn, &empty, "Pages", 0, now).await;
    let page = a_page(&world.database.conn, &context, "Only page", "text", 0, now).await;

    let alone = world.service.export(empty.id, ProjectExportCommand { blueprint: true, ..command(ExportFormat::Pdf) }).await.unwrap_err();
    let with_page = world.service.export(empty.id, ProjectExportCommand { blueprint: true, section_ids: vec![page.id], ..command(ExportFormat::Pdf) }).await.unwrap();

    assert!(alone.is_illegal_argument());
    assert_eq!(with_page.summary, "Only page");
    assert_eq!(recorded(&world).await.len(), 1);
}

#[tokio::test]
async fn one_item_is_named_and_summarised_by_its_title_in_each_format() {
    let world = world().await;

    let blueprint = world.service.export(world.vega.id, ProjectExportCommand { blueprint: true, ..command(ExportFormat::Pdf) }).await.unwrap();
    let page = world.service.export(world.vega.id, ProjectExportCommand { section_ids: vec![world.pages[2].id], ..command(ExportFormat::Docx) }).await.unwrap();
    let component = world.service.export(world.vega.id, ProjectExportCommand { component_ids: vec![world.components[0].id], ..command(ExportFormat::Pdf) }).await.unwrap();

    assert_eq!((blueprint.summary.as_str(), blueprint.file_name.as_str()), ("Blueprint", "vega-blueprint.pdf"));
    assert_eq!((page.summary.as_str(), page.file_name.as_str()), ("Release Plan", "vega-release-plan.docx"));
    assert_eq!((component.summary.as_str(), component.file_name.as_str()), ("api", "vega-api.pdf"));
    assert!(blueprint.status == ExportStatus::Done && blueprint.size_bytes > 0);
}

#[tokio::test]
async fn several_items_are_summarised_by_count_and_named_export() {
    let world = world().await;

    let view = world.service.export(world.vega.id, ProjectExportCommand {
        blueprint: true,
        section_ids: world.pages.iter().map(|page| page.id).collect(),
        component_ids: world.components.iter().map(|component| component.id).collect(),
        ..command(ExportFormat::Docx)
    }).await.unwrap();

    assert_eq!(view.summary, "Blueprint, 3 pages, 2 components");
    assert_eq!(view.file_name, "vega-export.docx");
    assert_eq!((view.project_title.as_str(), view.company_name.as_str()), ("VEGA", "Acme"));
}

#[tokio::test]
async fn a_diagram_image_reaches_the_document_and_a_missing_one_is_counted() {
    let world = world().await;
    let ticked = ProjectExportCommand { section_ids: vec![world.pages[1].id], ..command(ExportFormat::Pdf) };

    let drawn = plan_of(&world, ProjectExportCommand { diagrams: vec![diagram(FLOW, DIAGRAM_PNG)], ..ticked.clone() }).await.unwrap();
    let missing = plan_of(&world, ticked.clone()).await.unwrap();
    let exported = world.service.export(world.vega.id, ticked).await.unwrap();

    assert_eq!((drawn.document.diagrams.len(), drawn.diagrams_missing), (1, 0));
    assert_eq!((missing.document.diagrams.len(), missing.diagrams_missing), (0, 1));
    assert_eq!(exported.diagrams_missing, 1);
}

#[tokio::test]
async fn a_document_the_writer_refuses_is_recorded_failed_with_its_kind_and_still_returned_as_an_error() {
    let world = world().await;
    let ticked = ProjectExportCommand {
        section_ids: vec![world.pages[1].id],
        diagrams: vec![diagram(FLOW, b"not a png")],
        ..command(ExportFormat::Pdf)
    };

    let error = world.service.export(world.vega.id, ticked).await.unwrap_err();

    assert!(matches!(error, RekallError::Internal(_)));
    assert!(error.message().contains("Render error"));
    let rows = recorded(&world).await;
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].status, rows[0].failure_kind), (ExportStatus::Failed, Some(ExportFailureKind::Render)));
    assert!(rows[0].failure_message.as_deref().unwrap().starts_with("The PDF could not be written"));
    assert_eq!(rows[0].summary, "Setup");
    assert!(repo::project_export::file_of(&world.database.conn, rows[0].id).await.unwrap().is_none());
}

#[tokio::test]
async fn the_list_is_newest_first_with_the_names_a_page_groups_by() {
    let mut world = world().await;
    let first = world.service.export(world.vega.id, ProjectExportCommand { blueprint: true, ..command(ExportFormat::Pdf) }).await.unwrap();
    let later = Instant::parse("2026-11-25T13:00:00Z").unwrap();
    world.service.ctx.clock = Arc::new(move || later);
    let second = world.service.export(world.vega.id, ProjectExportCommand { blueprint: true, ..command(ExportFormat::Docx) }).await.unwrap();

    let listed = world.service.list().await.unwrap();

    assert_eq!(listed.iter().map(|view| view.id).collect::<Vec<_>>(), [second.id, first.id]);
    assert_eq!((listed[0].company_name.as_str(), listed[0].project_label.as_str()), ("Acme", "vega"));
}

#[tokio::test]
async fn the_file_of_a_done_export_downloads_and_that_of_a_failed_or_unknown_one_is_refused() {
    let world = world().await;
    let done = world.service.export(world.vega.id, ProjectExportCommand { blueprint: true, ..command(ExportFormat::Pdf) }).await.unwrap();
    world.service.export(world.vega.id, ProjectExportCommand {
        section_ids: vec![world.pages[1].id],
        diagrams: vec![diagram(FLOW, b"not a png")],
        ..command(ExportFormat::Pdf)
    }).await.unwrap_err();
    let failed = recorded(&world).await.into_iter().find(|row| row.status == ExportStatus::Failed).unwrap();

    let file = world.service.file(done.id).await.unwrap();

    assert_eq!((file.file_name.as_str(), file.media_type), ("vega-blueprint.pdf", "application/pdf"));
    assert!(file.bytes.starts_with(b"%PDF-"));
    assert!(matches!(world.service.file(failed.id).await.unwrap_err(), RekallError::NotFound(_)));
    assert!(matches!(world.service.file(Id::random()).await.unwrap_err(), RekallError::NotFound(_)));
}

#[tokio::test]
async fn a_word_file_downloads_with_the_word_media_type() {
    let world = world().await;
    let done = world.service.export(world.vega.id, ProjectExportCommand { blueprint: true, ..command(ExportFormat::Docx) }).await.unwrap();

    let file = world.service.file(done.id).await.unwrap();

    assert_eq!(file.media_type, "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
    assert!(file.bytes.starts_with(b"PK"));
}

#[tokio::test]
async fn deleting_an_export_removes_it_and_its_file() {
    let world = world().await;
    let done = world.service.export(world.vega.id, ProjectExportCommand { blueprint: true, ..command(ExportFormat::Pdf) }).await.unwrap();

    world.service.delete(done.id).await.unwrap();

    assert!(world.service.list().await.unwrap().is_empty());
    assert!(matches!(world.service.delete(done.id).await.unwrap_err(), RekallError::NotFound(_)));
}

#[tokio::test]
async fn clearing_deletes_what_matches_and_keeps_the_rest() {
    let world = world().await;
    let now = Instant::now();
    let atlas = a_project(&world.database.conn, world.company.id, "atlas", "AT", Some("Atlas."), now).await;
    world.service.export(world.vega.id, ProjectExportCommand { blueprint: true, ..command(ExportFormat::Pdf) }).await.unwrap();
    world.service.export(world.vega.id, ProjectExportCommand {
        section_ids: vec![world.pages[1].id],
        diagrams: vec![diagram(FLOW, b"not a png")],
        ..command(ExportFormat::Pdf)
    }).await.unwrap_err();
    world.service.export(atlas.id, ProjectExportCommand { blueprint: true, ..command(ExportFormat::Pdf) }).await.unwrap();

    let failed = world.service.clear(ProjectExportClearCommand { status: Some(ExportStatus::Failed), ..Default::default() }).await.unwrap();
    let of_atlas = world.service.clear(ProjectExportClearCommand { project_id: Some(atlas.id), ..Default::default() }).await.unwrap();

    assert_eq!((failed, of_atlas), (1, 1));
    assert_eq!(world.service.list().await.unwrap().len(), 1);
    let of_company = world.service.clear(ProjectExportClearCommand { company_id: Some(world.company.id), ..Default::default() }).await.unwrap();
    assert_eq!(of_company, 1);
    assert!(world.service.list().await.unwrap().is_empty());
}

#[tokio::test]
async fn clearing_with_no_criterion_or_an_unknown_company_is_refused() {
    let world = world().await;
    world.service.export(world.vega.id, ProjectExportCommand { blueprint: true, ..command(ExportFormat::Pdf) }).await.unwrap();

    let nothing = world.service.clear(ProjectExportClearCommand::default()).await.unwrap_err();
    let unknown = world.service.clear(ProjectExportClearCommand { company_id: Some(Id::random()), ..Default::default() }).await.unwrap_err();

    assert!(nothing.is_illegal_argument());
    assert!(matches!(unknown, RekallError::NotFound(_)));
    assert_eq!(world.service.list().await.unwrap().len(), 1);
    let _ = (&world.guides, &world.decisions);
}

fn jpeg(width: u32, height: u32) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    RgbImage::from_pixel(width, height, Rgb([200, 120, 40])).write_to(&mut out, ImageFormat::Jpeg).unwrap();
    out.into_inner()
}

/// A project whose blueprint shows a JPEG, an SVG, a picture whose file is gone and one on the web.
async fn a_project_with_pictures(world: &World) -> project::Model {
    world.store.keep("acme/atlas/blueprint/photo-00000001.jpg", jpeg(60, 40));
    world.store.keep("acme/atlas/blueprint/flow-00000002.svg", br#"<svg xmlns="http://www.w3.org/2000/svg" width="30" height="10"/>"#.to_vec());
    let blueprint = "![Photo](/api/images/acme/atlas/blueprint/photo-00000001.jpg#center)\n\n\
        ![Flow](/api/images/acme/atlas/blueprint/flow-00000002.svg)\n\n\
        ![Gone](/api/images/acme/atlas/blueprint/gone-00000003.png)\n\n\
        ![Web](https://example.com/logo.png)";
    a_project(&world.database.conn, world.company.id, "atlas", "AT", Some(blueprint), Instant::now()).await
}

#[tokio::test]
async fn the_pictures_of_the_ticked_texts_reach_the_document_as_png_and_a_missing_one_is_left_out() {
    let world = world().await;
    let atlas = a_project_with_pictures(&world).await;
    let ticked = ProjectExportCommand { blueprint: true, ..command(ExportFormat::Pdf) };
    let plan = plan_for(&world, atlas.id, ticked).await.unwrap();

    let pictures = world.service.pictures_of(&plan).await;

    let handed: Vec<(&str, u32)> = pictures.iter().map(|picture| (picture.address.as_str(), picture.natural_width)).collect();
    assert_eq!(handed, [("/api/images/acme/atlas/blueprint/photo-00000001.jpg", 60), ("/api/images/acme/atlas/blueprint/flow-00000002.svg", 30)]);
    assert!(pictures.iter().all(|picture| image::guess_format(&picture.png).unwrap() == ImageFormat::Png));
}

#[tokio::test]
async fn an_export_with_a_missing_picture_is_written_in_each_format() {
    let world = world().await;
    let atlas = a_project_with_pictures(&world).await;

    for format in [ExportFormat::Pdf, ExportFormat::Docx] {
        let view = world.service.export(atlas.id, ProjectExportCommand { blueprint: true, ..command(format) }).await.unwrap();

        assert_eq!(view.status, ExportStatus::Done, "{format}");
    }
}
