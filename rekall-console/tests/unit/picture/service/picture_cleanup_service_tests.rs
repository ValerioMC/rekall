use rekall_core::picture::{InMemoryPictureStore, PictureCleanupState, PictureCleanupStreamEvent};
use rekall_core::{EventBus, PublishedEvent};
use rekall_kernel::{Id, Instant};
use rekall_store::{company, document, project, task, DocumentContextMode, ProjectStatus};
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel};
use tokio::sync::broadcast::Receiver;

use super::*;

const NOW: &str = "2026-03-10T12:00:00Z";
const LONG_AGO: &str = "2026-01-01T00:00:00Z";

struct World {
    service: PictureCleanupService,
    store: Arc<InMemoryPictureStore>,
    ctx: Ctx,
    events: Receiver<PublishedEvent>,
    task: Id,
    _database: rekall_store::Database,
}

async fn a_task(db: &DatabaseConnection, now: Instant) -> Id {
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "web".into(), title: "Web".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false,
        task_key: "WEB".into(), company_id: company.id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    let task = task::Model::new("web-12".into(), "Login".into(), project.id, now);
    task.clone().into_active_model().insert(db).await.unwrap();
    task.id
}

async fn world() -> World {
    world_over(InMemoryPictureStore::in_folder()).await
}

async fn world_over(store: InMemoryPictureStore) -> World {
    let database = rekall_store::open_temporary().await.unwrap();
    let mut ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::parse(NOW).unwrap();
    ctx.clock = Arc::new(move || now);
    let task = a_task(&database.conn, now).await;
    let events = ctx.events.subscribe();
    let store = Arc::new(store);
    World { service: PictureCleanupService::new(ctx.clone(), store.clone()), store, ctx, events, task, _database: database }
}

fn path(name: &str) -> PicturePath {
    PicturePath::parse(&format!("acme/web/web-12/{name}.png")).unwrap()
}

impl World {
    fn an_old_picture(&self, name: &str) {
        self.store.put_modified_at(&path(name), b"pixels", Instant::parse(LONG_AGO).unwrap());
    }

    async fn a_note(&self, position: i32, body: &str) {
        let now = self.ctx.now();
        let note = document::Model {
            id: Id::random(), task_id: self.task, position, title: format!("note-{position}"), kind: "notes".into(), body_markdown: body.into(),
            source_path: None, context_mode: DocumentContextMode::Full, created_at: now, updated_at: now,
        };
        note.into_active_model().insert(&self.ctx.db).await.unwrap();
    }

    /// Every frame up to the one that ends the run under way.
    async fn frames_to_the_end(&mut self) -> Vec<PictureCleanupStreamEvent> {
        let mut frames = Vec::new();
        loop {
            let event = self.events.recv().await.unwrap();
            let frame = event.downcast::<PictureCleanupStreamEvent>().unwrap().clone();
            let state = frame.state;
            frames.push(frame);
            if !matches!(state, PictureCleanupState::Searching | PictureCleanupState::Deleting) {
                return frames;
            }
        }
    }

    async fn searched(&mut self) -> PictureCleanupStatusView {
        self.service.start().unwrap();
        self.frames_to_the_end().await;
        self.service.status()
    }

    async fn deleted(&mut self) -> PictureCleanupStatusView {
        self.service.delete().unwrap();
        self.frames_to_the_end().await;
        self.service.status()
    }

    async fn holds(&self, name: &str) -> bool {
        self.store.read(&path(name)).await.is_ok()
    }
}

fn listed(status: &PictureCleanupStatusView) -> Vec<String> {
    match status {
        PictureCleanupStatusView::Ready { pictures } => pictures.iter().map(|unused| unused.picture.relative().to_string()).collect(),
        other => panic!("no list in {other:?}"),
    }
}

fn finished(removed: usize, freed_bytes: u64, kept: usize, not_deleted: usize) -> PictureCleanupStatusView {
    PictureCleanupStatusView::Finished { removed, freed_bytes, kept, not_deleted }
}

#[tokio::test]
async fn only_an_old_picture_no_text_shows_is_listed_and_deleted() {
    let mut world = world().await;
    world.an_old_picture("unused");
    world.an_old_picture("used");
    world.store.put_modified_at(&path("young"), b"pixels", Instant::parse("2026-03-10T11:00:00Z").unwrap());
    world.a_note(0, "![used](/api/images/acme/web/web-12/used.png)").await;

    let found = world.searched().await;
    assert_eq!(found, PictureCleanupStatusView::Ready { pictures: vec![UnusedPictureView { picture: path("unused"), size_bytes: 6 }] });

    assert_eq!(world.deleted().await, finished(1, 6, 0, 0));
    assert!(!world.holds("unused").await);
    assert!(world.holds("used").await && world.holds("young").await);
}

#[tokio::test]
async fn a_link_past_the_first_page_of_rows_is_found() {
    let mut world = world().await;
    world.an_old_picture("late");
    for position in 0..PAGE_ROWS as i32 {
        world.a_note(position, "nothing").await;
    }
    world.a_note(PAGE_ROWS as i32, "![late](/api/images/acme/web/web-12/late.png)").await;

    assert!(listed(&world.searched().await).is_empty());
}

#[tokio::test]
async fn a_search_announces_each_new_percentage_and_ends_with_the_count() {
    let mut world = world().await;
    world.an_old_picture("unused");

    world.service.start().unwrap();
    let frames = world.frames_to_the_end().await;

    assert_eq!(frames[0], PictureCleanupStreamEvent { state: PictureCleanupState::Searching, percentage: 0, count: 0 });
    assert_eq!(frames.last(), Some(&PictureCleanupStreamEvent { state: PictureCleanupState::Ready, percentage: 100, count: 1 }));
    assert!(frames.windows(2).all(|pair| pair[0] != pair[1] && (pair[1].percentage >= pair[0].percentage)), "{frames:?}");
}

#[tokio::test]
async fn a_picture_linked_after_the_search_is_kept() {
    let mut world = world().await;
    world.an_old_picture("pasted-back");
    world.an_old_picture("unused");
    world.searched().await;

    world.a_note(0, "![back](/api/images/acme/web/web-12/pasted-back.png)").await;

    assert_eq!(world.deleted().await, finished(1, 6, 1, 0));
    assert!(world.holds("pasted-back").await);
}

#[tokio::test]
async fn a_picture_written_again_after_the_search_is_kept_and_one_already_gone_counts_nowhere() {
    let mut world = world().await;
    world.an_old_picture("rewritten");
    world.an_old_picture("gone");
    world.searched().await;

    world.store.put_modified_at(&path("rewritten"), b"pixels", world.ctx.now());
    world.store.delete(&path("gone")).await.unwrap();

    assert_eq!(world.deleted().await, finished(0, 0, 1, 0));
    assert!(world.holds("rewritten").await);
}

#[tokio::test]
async fn a_picture_that_cannot_be_deleted_is_counted_and_the_others_go() {
    let mut world = world().await;
    world.an_old_picture("a-locked");
    world.an_old_picture("b-free");
    world.store.refuse_deleting(&path("a-locked"));
    world.searched().await;

    assert_eq!(world.deleted().await, finished(1, 6, 0, 1));
    assert!(!world.holds("b-free").await);
}

#[tokio::test]
async fn a_search_that_cannot_read_the_texts_fails() {
    let mut world = world().await;
    world.an_old_picture("unused");
    world.ctx.db.clone().close().await.unwrap();

    assert!(matches!(world.searched().await, PictureCleanupStatusView::Failed { .. }));
}

#[tokio::test]
async fn a_deletion_that_cannot_read_the_texts_deletes_nothing() {
    let mut world = world().await;
    world.an_old_picture("unused");
    world.searched().await;
    world.ctx.db.clone().close().await.unwrap();

    assert!(matches!(world.deleted().await, PictureCleanupStatusView::Failed { .. }));
    assert!(world.holds("unused").await);
}

#[tokio::test]
async fn calls_outside_their_state_are_refused() {
    let mut world = world().await;
    let refused = |outcome: Result<()>| matches!(outcome, Err(RekallError::Conflict(_)));
    assert!(refused(world.service.delete()));
    assert!(refused(world.service.discard()));

    world.service.start().unwrap();
    assert!(refused(world.service.start()));
    assert!(refused(world.service.delete()));
    assert!(refused(world.service.discard()));
    world.frames_to_the_end().await;

    assert!(refused(world.service.start()));
    world.service.delete().unwrap();
    assert!(refused(world.service.start()));
    assert!(refused(world.service.delete()));
    assert!(refused(world.service.discard()));
}

#[tokio::test]
async fn a_discarded_list_goes_back_to_idle_and_deletes_nothing() {
    let mut world = world().await;
    world.an_old_picture("unused");
    world.searched().await;

    world.service.discard().unwrap();

    assert_eq!(world.service.status(), PictureCleanupStatusView::Idle);
    assert!(world.holds("unused").await);
    assert!(world.service.start().is_ok());
}

#[tokio::test]
async fn a_database_that_is_not_a_file_is_refused() {
    let world = world_over(InMemoryPictureStore::new()).await;

    for outcome in [world.service.start(), world.service.delete(), world.service.discard()] {
        assert!(matches!(outcome, Err(RekallError::Conflict(_))));
    }
    assert_eq!(world.service.status(), PictureCleanupStatusView::Idle);
}
