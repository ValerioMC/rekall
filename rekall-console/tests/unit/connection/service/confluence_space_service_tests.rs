use async_trait::async_trait;
use rekall_core::connection::{AtlassianCredential, AtlassianFailure, AtlassianSite, ConfluencePage, ConfluenceSpace, InMemorySecretStore, Secret};
use rekall_core::{Ctx, EventBus};
use rekall_kernel::Instant;
use rekall_store::repository as repo;
use rekall_store::{company, external_connection, project, project_connection, ProjectStatus};
use rekall_core::{in_read, Services};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;
use crate::catalog::ProjectService;
use crate::section::SectionWriteOutcome;

struct FakeConfluence {
    space: Option<ConfluenceSpace>,
    pages: Vec<ConfluencePage>,
    failure: Option<AtlassianFailure>,
}

#[async_trait]
impl ConfluenceSpaces for FakeConfluence {
    async fn space(&self, _site: &AtlassianSite, credential: &AtlassianCredential, _space_key: &str) -> std::result::Result<Option<ConfluenceSpace>, AtlassianFailure> {
        assert_eq!(credential.email(), Some("ada@example.com"));
        match &self.failure {
            Some(failure) => Err(failure.clone()),
            None => Ok(self.space.clone()),
        }
    }

    async fn spaces(&self, _site: &AtlassianSite, _credential: &AtlassianCredential, _query: &str, _limit: u16) -> std::result::Result<Vec<ConfluenceSpace>, AtlassianFailure> {
        Ok(Vec::new())
    }

    async fn pages(&self, _site: &AtlassianSite, _credential: &AtlassianCredential, _space_key: &str, _limit: u16) -> std::result::Result<Vec<ConfluencePage>, AtlassianFailure> {
        Ok(self.pages.clone())
    }
}

struct World {
    service: ConfluenceSpaceService,
    ctx: Ctx,
    sections: ProjectSectionCatalogService,
    project_id: Id,
    binding_id: Id,
}

fn page(id: &str, title: &str, markdown: &str) -> ConfluencePage {
    ConfluencePage { id: id.into(), title: title.into(), labels: vec![], markdown: markdown.into(), url: format!("https://wiki/{id}") }
}

fn space() -> ConfluenceSpace {
    ConfluenceSpace { key: "ENG".into(), name: "Engineering".into(), description: Some("How we build".into()), home_page_id: Some("1".into()) }
}

async fn world(kind: ConnectionKind, target: Option<&str>, failure: Option<AtlassianFailure>) -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project = project::Model {
        id: Id::random(),
        label: "app".into(),
        title: "App".into(),
        status: ProjectStatus::Active,
        icon: "folder".into(),
        description: None,
        blueprint_markdown: None,
        repo_folder: None,
        auto_commit: false, main_branch: None, pull_request_flow: false, task_key: "AP".into(),
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let mut connection = external_connection::Model::new("wiki".into(), "Wiki".into(), kind, Some("https://example.atlassian.net/wiki".into()), now);
    connection.secret_set_at = Some(now);
    connection.clone().into_active_model().insert(&database.conn).await.unwrap();
    let binding = project_connection::Model::new(project.id, connection.id, connection.label.clone(), target.map(str::to_string), now);
    binding.clone().into_active_model().insert(&database.conn).await.unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());
    secrets.put(&connection.secret_ref, &Secret::new("ada@example.com:atl-secret")).unwrap();
    let services = Services::new(ctx.clone(), Arc::new(rekall_core::check::ScriptedCheckRunner::new()));
    let projects = ProjectService::new(ctx.clone(), services.repositories, rekall_core::worktree::PullRequestFlowService);
    let sections = ProjectSectionCatalogService::new(ctx.clone());
    let confluence = Arc::new(FakeConfluence {
        space: Some(space()),
        pages: vec![page("1", "Home", "Welcome."), page("2", "Release process", "Steps"), page("3", "System design", "Boxes")],
        failure,
    });
    let service = ConfluenceSpaceService::new(ctx.clone(), ProjectConnectionService::new(ctx.clone()), projects, sections.clone(), secrets, confluence);
    World { service, ctx, sections, project_id: project.id, binding_id: binding.id }
}

async fn stored_project(world: &World) -> project::Model {
    read_project(world).await.unwrap()
}

async fn read_project(world: &World) -> Result<project::Model> {
    in_read!(&world.ctx, |tx| repo::project::require(tx.db(), world.project_id).await)
}

async fn titles(world: &World) -> Vec<String> {
    read_titles(world).await.unwrap()
}

async fn read_titles(world: &World) -> Result<Vec<String>> {
    in_read!(&world.ctx, |tx| Ok::<_, RekallError>(repo::project_section::list_of_project(tx.db(), world.project_id).await?.into_iter().map(|section| section.title).collect()))
}

#[tokio::test]
async fn a_space_fills_the_blueprint_the_description_and_a_section_per_concept() {
    let world = world(ConnectionKind::Confluence, Some("ENG"), None).await;

    let imported = world.service.import(world.project_id, world.binding_id, ConfluenceImportCommand::default()).await.unwrap();

    let project = stored_project(&world).await;
    assert_eq!(project.description.as_deref(), Some("How we build"));
    assert_eq!(project.blueprint_markdown.as_deref(), Some("Welcome.\n\nSource: [Home](https://wiki/1)"));
    assert_eq!(titles(&world).await, ["Architecture", "Deployment"]);
    assert_eq!((imported.pages_read, imported.description_written, imported.blueprint_written), (3, true, true));
    assert!(imported.sections.iter().all(|section| section.outcome == SectionWriteOutcome::Created));
}

#[tokio::test]
async fn a_second_import_keeps_what_the_project_has_unless_asked_to_replace_it() {
    let world = world(ConnectionKind::Confluence, Some("ENG"), None).await;
    world.service.import(world.project_id, world.binding_id, ConfluenceImportCommand::default()).await.unwrap();
    let edited = world.sections.create(world.project_id, crate::section::ProjectSectionCommand { title: Some("Notes".into()), body_markdown: Some("mine".into()) }).await.unwrap();

    let kept = world.service.import(world.project_id, world.binding_id, ConfluenceImportCommand::default()).await.unwrap();
    let replaced = world.service.import(world.project_id, world.binding_id, ConfluenceImportCommand { replace: true }).await.unwrap();

    assert!(!kept.description_written && !kept.blueprint_written);
    assert!(kept.sections.iter().all(|section| section.outcome == SectionWriteOutcome::Kept));
    assert!(replaced.description_written);
    assert!(replaced.sections.iter().all(|section| section.outcome == SectionWriteOutcome::Replaced));
    assert_eq!(titles(&world).await, ["Architecture", "Deployment", edited.title.as_str()]);
}

#[tokio::test]
async fn a_binding_that_is_not_confluence_or_names_no_space_is_refused_before_confluence_is_asked() {
    let jira = world(ConnectionKind::Jira, Some("ENG"), None).await;
    let without_key = world(ConnectionKind::Confluence, None, None).await;

    assert!(jira.service.import(jira.project_id, jira.binding_id, ConfluenceImportCommand::default()).await.unwrap_err().is_illegal_argument());
    assert!(without_key.service.import(without_key.project_id, without_key.binding_id, ConfluenceImportCommand::default()).await.unwrap_err().is_illegal_argument());
}

#[tokio::test]
async fn a_credential_confluence_rejects_is_a_conflict_and_the_project_is_untouched() {
    let world = world(ConnectionKind::Confluence, Some("ENG"), Some(AtlassianFailure::Rejected { status: 403 })).await;

    let error = world.service.import(world.project_id, world.binding_id, ConfluenceImportCommand::default()).await.unwrap_err();

    assert!(matches!(error, RekallError::Conflict(_)));
    assert_eq!(stored_project(&world).await.description, None);
    assert!(titles(&world).await.is_empty());
}
