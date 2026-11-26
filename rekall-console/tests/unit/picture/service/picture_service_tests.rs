use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use async_trait::async_trait;
use rekall_core::EventBus;
use rekall_kernel::{Id, Instant};
use rekall_store::{company, project, project_component, project_section, section_context, task, ProjectStatus};
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel};

use super::*;
use crate::picture::PictureOwner;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n pixels";
const LOGIN_FLOW: &str = "/home/me/Desktop/Login Flow.png";

#[derive(Default)]
struct FakeStore {
    sources: Mutex<HashMap<PathBuf, Vec<u8>>>,
    kept: Mutex<HashMap<String, Vec<u8>>>,
    writes: Mutex<usize>,
}

#[async_trait]
impl PictureStore for FakeStore {
    async fn read_source(&self, source: &Path, limit_bytes: u64) -> Result<Vec<u8>> {
        let bytes = self.sources.lock().unwrap().get(source).cloned().ok_or_else(|| RekallError::illegal("not a file"))?;
        if bytes.len() as u64 > limit_bytes {
            return Err(RekallError::illegal("too large"));
        }
        Ok(bytes)
    }

    async fn write(&self, path: &PicturePath, bytes: &[u8]) -> Result<()> {
        *self.writes.lock().unwrap() += 1;
        self.kept.lock().unwrap().insert(path.relative().to_string(), bytes.to_vec());
        Ok(())
    }

    async fn read(&self, path: &PicturePath) -> Result<Vec<u8>> {
        self.kept.lock().unwrap().get(path.relative()).cloned().ok_or_else(|| RekallError::not_found_msg("no picture"))
    }
}

struct World {
    service: PictureService,
    store: Arc<FakeStore>,
    project: Id,
    task: Id,
    page: Id,
    component: Id,
    _database: rekall_store::Database,
}

async fn a_company(db: &DatabaseConnection, name: &str, now: Instant) -> company::Model {
    let company = company::Model { id: Id::random(), name: name.into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    company
}

async fn a_project(db: &DatabaseConnection, company_id: Id, now: Instant) -> project::Model {
    let project = project::Model {
        id: Id::random(), label: "web".into(), title: "Web".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false,
        task_key: "WEB".into(), company_id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    project
}

async fn a_task(db: &DatabaseConnection, project: &project::Model, now: Instant) -> task::Model {
    let mut task = task::Model::new("web-12".into(), "Login".into(), project.id, now);
    task.assign_code(&project.task_key, 12);
    task.clone().into_active_model().insert(db).await.unwrap();
    task
}

async fn a_page(db: &DatabaseConnection, project_id: Id, now: Instant) -> project_section::Model {
    let context = section_context::Model { id: Id::random(), project_id, title: "General".into(), position: 0, created_at: now, updated_at: now };
    context.clone().into_active_model().insert(db).await.unwrap();
    let page = project_section::Model {
        id: Id::random(), project_id, context_id: context.id, title: "Release Plan".into(), body_markdown: String::new(),
        position: 0, created_at: now, updated_at: now,
    };
    page.clone().into_active_model().insert(db).await.unwrap();
    page
}

async fn a_component(db: &DatabaseConnection, project_id: Id, now: Instant) -> project_component::Model {
    let component = project_component::Model {
        id: Id::random(), project_id, name: "API Server".into(), folder: "/work/api".into(), summary: String::new(),
        description: String::new(), position: 0, created_at: now, updated_at: now,
    };
    component.clone().into_active_model().insert(db).await.unwrap();
    component
}

async fn world_of(company_name: &str) -> World {
    let database = rekall_store::open_temporary().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::parse("2026-11-26T12:00:00Z").unwrap();
    let db = &database.conn;
    let company = a_company(db, company_name, now).await;
    let project = a_project(db, company.id, now).await;
    let task = a_task(db, &project, now).await;
    let page = a_page(db, project.id, now).await;
    let component = a_component(db, project.id, now).await;
    let store = Arc::new(FakeStore::default());
    store.sources.lock().unwrap().insert(LOGIN_FLOW.into(), PNG.to_vec());
    World {
        service: PictureService::new(ctx, store.clone()),
        store,
        project: project.id,
        task: task.id,
        page: page.id,
        component: component.id,
        _database: database,
    }
}

async fn world() -> World {
    world_of("Acme").await
}

fn import_of(source: &str, owner: PictureOwner) -> PictureImportCommand {
    PictureImportCommand { source_path: source.into(), owner }
}

async fn folder_of(world: &World, owner: PictureOwner) -> String {
    let url = world.service.import(import_of(LOGIN_FLOW, owner)).await.unwrap().url;
    url.rsplit_once('/').unwrap().0.to_string()
}

#[tokio::test]
async fn each_owner_keeps_its_pictures_in_a_folder_of_its_own() {
    let world = world().await;

    assert_eq!(folder_of(&world, PictureOwner::Task(world.task)).await, "/api/images/acme/web/web-12");
    assert_eq!(folder_of(&world, PictureOwner::Blueprint(world.project)).await, "/api/images/acme/web/blueprint");
    assert_eq!(folder_of(&world, PictureOwner::Page(world.page)).await, "/api/images/acme/web/pages/release-plan");
    assert_eq!(folder_of(&world, PictureOwner::Component(world.component)).await, "/api/images/acme/web/components/api-server");
}

#[tokio::test]
async fn a_company_name_with_nothing_usable_falls_back_to_the_start_of_its_id() {
    let world = world_of("***").await;

    let folder = folder_of(&world, PictureOwner::Task(world.task)).await;

    let company = folder.strip_prefix("/api/images/").unwrap().split('/').next().unwrap().to_string();
    assert!(company.len() == 8 && company.chars().all(|c| c.is_ascii_hexdigit()), "{company}");
}

#[tokio::test]
async fn an_imported_picture_is_written_and_read_back_by_its_address() {
    let world = world().await;

    let url = world.service.import(import_of(LOGIN_FLOW, PictureOwner::Task(world.task))).await.unwrap().url;

    assert!(url.starts_with("/api/images/acme/web/web-12/login-flow-") && url.ends_with(".png"), "{url}");
    let file = world.service.read(url.strip_prefix("/api/images/").unwrap()).await.unwrap();
    assert_eq!(file, PictureFileView { bytes: PNG.to_vec(), format: rekall_core::picture::PictureFormat::Png });
}

#[tokio::test]
async fn the_same_bytes_twice_give_one_address_and_are_written_again() {
    let world = world().await;
    let command = import_of(LOGIN_FLOW, PictureOwner::Task(world.task));

    let first = world.service.import(command.clone()).await.unwrap();
    let second = world.service.import(command).await.unwrap();

    assert_eq!(first, second);
    assert_eq!(*world.store.writes.lock().unwrap(), 2);
    assert_eq!(world.store.kept.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn a_file_that_is_not_a_picture_is_refused_and_not_written() {
    let world = world().await;
    world.store.sources.lock().unwrap().insert("/home/me/notes.png".into(), b"only text".to_vec());

    let refusal = world.service.import(import_of("/home/me/notes.png", PictureOwner::Task(world.task))).await.unwrap_err();

    assert!(matches!(refusal, RekallError::IllegalArgument(_)));
    assert_eq!(*world.store.writes.lock().unwrap(), 0);
}

#[tokio::test]
async fn a_picture_past_ten_megabytes_is_refused() {
    let world = world().await;
    let mut huge = PNG.to_vec();
    huge.resize(10 * 1024 * 1024 + 1, 0);
    world.store.sources.lock().unwrap().insert("/home/me/huge.png".into(), huge);

    let refusal = world.service.import(import_of("/home/me/huge.png", PictureOwner::Task(world.task))).await.unwrap_err();

    assert_eq!(refusal.message(), "too large");
}

#[tokio::test]
async fn an_owner_that_does_not_exist_is_not_found() {
    let world = world().await;
    let unknown = Id::random();

    for owner in [PictureOwner::Task(unknown), PictureOwner::Blueprint(unknown), PictureOwner::Page(unknown), PictureOwner::Component(unknown)] {
        let refusal = world.service.import(import_of(LOGIN_FLOW, owner)).await.unwrap_err();
        assert!(matches!(refusal, RekallError::NotFound(_)), "{owner:?}: {refusal}");
    }
    assert_eq!(*world.store.writes.lock().unwrap(), 0);
}

#[tokio::test]
async fn reading_a_missing_picture_or_something_that_is_not_one_is_not_found() {
    let world = world().await;

    for path in ["acme/web/web-12/gone-00000000.png", "acme/web/../../rekall.db", "rekall.db"] {
        assert!(matches!(world.service.read(path).await, Err(RekallError::NotFound(_))), "{path}");
    }
}

#[tokio::test]
async fn a_thumbnail_is_drawn_from_the_chosen_file_and_nothing_is_kept() {
    let world = world().await;
    let mut photo = std::io::Cursor::new(Vec::new());
    image::RgbImage::new(900, 600).write_to(&mut photo, image::ImageFormat::Png).unwrap();
    world.store.sources.lock().unwrap().insert("/home/me/photo.png".into(), photo.into_inner());

    let thumbnail = world.service.thumbnail("/home/me/photo.png").await.unwrap();

    let drawn = image::load_from_memory(&thumbnail.bytes).unwrap();
    assert_eq!((drawn.width(), drawn.height()), (320, 213));
    assert_eq!(*world.store.writes.lock().unwrap(), 0);
}

#[tokio::test]
async fn a_thumbnail_is_refused_for_what_an_import_refuses() {
    let world = world().await;
    let mut huge = PNG.to_vec();
    huge.resize(10 * 1024 * 1024 + 1, 0);
    world.store.sources.lock().unwrap().insert("/home/me/huge.png".into(), huge);
    world.store.sources.lock().unwrap().insert("/home/me/notes.png".into(), b"only text".to_vec());

    for source in ["/home/me/huge.png", "/home/me/notes.png", "/home/me/gone.png"] {
        let refusal = world.service.thumbnail(source).await.unwrap_err();
        assert!(matches!(refusal, RekallError::IllegalArgument(_)), "{source}: {refusal}");
    }
}
