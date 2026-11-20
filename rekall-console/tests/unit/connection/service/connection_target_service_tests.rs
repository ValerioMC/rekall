use std::sync::Mutex;

use async_trait::async_trait;
use rekall_core::connection::{ScriptedGitlabNamespaces, AtlassianCredential, AtlassianFailure, AtlassianSite, ConfluencePage, ConfluenceSpace, InMemorySecretStore, JiraIssue, JiraProject, Secret};
use rekall_core::{Ctx, EventBus};
use rekall_kernel::Instant;
use rekall_store::{company, external_connection, project, ProjectStatus};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

struct FakeJira {
    keys: Vec<&'static str>,
    asked: Mutex<Vec<String>>,
}

#[async_trait]
impl JiraIssues for FakeJira {
    async fn search(&self, _site: &AtlassianSite, _credential: &AtlassianCredential, _jql: &str, _limit: u16) -> std::result::Result<Vec<JiraIssue>, AtlassianFailure> {
        Ok(Vec::new())
    }

    async fn projects(&self, _site: &AtlassianSite, _credential: &AtlassianCredential, query: &str, _limit: u16) -> std::result::Result<Vec<JiraProject>, AtlassianFailure> {
        self.asked.lock().unwrap().push(query.to_string());
        Ok(self.keys.iter().map(|key| JiraProject { key: key.to_string(), name: format!("{key} project") }).collect())
    }

    async fn project(&self, _site: &AtlassianSite, _credential: &AtlassianCredential, key: &str) -> std::result::Result<Option<JiraProject>, AtlassianFailure> {
        Ok(self.keys.contains(&key).then(|| JiraProject { key: key.to_string(), name: format!("{key} project") }))
    }
}

struct FakeConfluence {
    keys: Vec<&'static str>,
    failure: Option<AtlassianFailure>,
}

#[async_trait]
impl ConfluenceSpaces for FakeConfluence {
    async fn space(&self, _site: &AtlassianSite, _credential: &AtlassianCredential, space_key: &str) -> std::result::Result<Option<ConfluenceSpace>, AtlassianFailure> {
        if let Some(failure) = &self.failure {
            return Err(failure.clone());
        }
        Ok(self.keys.contains(&space_key).then(|| ConfluenceSpace { key: space_key.to_string(), name: format!("{space_key} space"), description: None, home_page_id: None }))
    }

    async fn spaces(&self, _site: &AtlassianSite, _credential: &AtlassianCredential, _query: &str, _limit: u16) -> std::result::Result<Vec<ConfluenceSpace>, AtlassianFailure> {
        Ok(self.keys.iter().map(|key| ConfluenceSpace { key: key.to_string(), name: format!("{key} space"), description: None, home_page_id: None }).collect())
    }

    async fn pages(&self, _site: &AtlassianSite, _credential: &AtlassianCredential, _space_key: &str, _limit: u16) -> std::result::Result<Vec<ConfluencePage>, AtlassianFailure> {
        Ok(Vec::new())
    }
}

struct World {
    service: ConnectionTargetService,
    bindings: ProjectConnectionService,
    jira: Arc<FakeJira>,
    project_id: Id,
    connection_id: Id,
}

async fn world(kind: ConnectionKind, credential_stored: bool, confluence_failure: Option<AtlassianFailure>) -> World {
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
    let mut connection = external_connection::Model::new("site".into(), "Site".into(), kind, Some("https://example.atlassian.net".into()), now);
    connection.secret_set_at = credential_stored.then_some(now);
    connection.clone().into_active_model().insert(&database.conn).await.unwrap();
    rekall_store::connection_company::Model { connection_id: connection.id, company_id: company.id }.into_active_model().insert(&database.conn).await.unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());
    secrets.put(&connection.secret_ref, &Secret::new("ada@example.com:atl-secret")).unwrap();
    let jira = Arc::new(FakeJira { keys: vec!["ENG", "OPS"], asked: Mutex::new(Vec::new()) });
    let confluence = Arc::new(FakeConfluence { keys: vec!["DOCS"], failure: confluence_failure });
    let bindings = ProjectConnectionService::new(ctx.clone());
    let gitlab = GitlabTargetService::new(ctx.clone(), secrets.clone(), Arc::new(ScriptedGitlabNamespaces::new()));
    let service = ConnectionTargetService::new(ctx, bindings.clone(), secrets, jira.clone(), confluence, gitlab);
    World { service, bindings, jira, project_id: project.id, connection_id: connection.id }
}

fn target(target: &str) -> ProjectConnectionCommand {
    ProjectConnectionCommand { target: Some(target.into()), ..ProjectConnectionCommand::default() }
}

#[tokio::test]
async fn a_jira_project_that_exists_is_bound_in_the_case_it_was_typed() {
    let world = world(ConnectionKind::Jira, true, None).await;

    let bound = world.service.bind(world.project_id, world.connection_id, target("eng")).await.unwrap();

    assert_eq!(bound.target.as_deref(), Some("eng"));
}

#[tokio::test]
async fn a_jira_project_the_site_lacks_is_refused_and_nothing_is_bound() {
    let world = world(ConnectionKind::Jira, true, None).await;

    let error = world.service.bind(world.project_id, world.connection_id, target("NOPE")).await.unwrap_err();

    assert!(error.is_illegal_argument());
    assert!(error.message().contains("Jira has no project 'NOPE'"));
    assert!(world.bindings.list(world.project_id).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_confluence_space_is_checked_when_bound_and_when_retargeted() {
    let world = world(ConnectionKind::Confluence, true, None).await;

    let missing = world.service.bind(world.project_id, world.connection_id, target("NOPE")).await.unwrap_err();
    let bound = world.service.bind(world.project_id, world.connection_id, target("DOCS")).await.unwrap();
    let retargeted = world.service.update(world.project_id, bound.id, target("OTHER")).await.unwrap_err();

    assert!(missing.message().contains("Confluence has no space 'NOPE'"));
    assert!(retargeted.is_illegal_argument());
    assert_eq!(world.bindings.list(world.project_id).await.unwrap()[0].target.as_deref(), Some("DOCS"));
}

#[tokio::test]
async fn a_binding_without_a_target_is_kept_without_asking_the_site() {
    let world = world(ConnectionKind::Jira, false, None).await;

    let bound = world.service.bind(world.project_id, world.connection_id, ProjectConnectionCommand::default()).await.unwrap();

    assert_eq!(bound.target, None);
}

#[tokio::test]
async fn a_target_cannot_be_checked_without_a_stored_credential() {
    let world = world(ConnectionKind::Jira, false, None).await;

    let error = world.service.bind(world.project_id, world.connection_id, target("ENG")).await.unwrap_err();

    assert!(error.message().contains("No credential is stored"));
}

#[tokio::test]
async fn a_site_that_rejects_the_credential_is_answered_as_a_conflict() {
    let world = world(ConnectionKind::Confluence, true, Some(AtlassianFailure::Rejected { status: 401 })).await;

    let error = world.service.bind(world.project_id, world.connection_id, target("DOCS")).await.unwrap_err();

    assert!(matches!(error, RekallError::Conflict(_)));
    assert!(error.message().contains("HTTP 401"));
}

#[tokio::test]
async fn suggestions_are_the_projects_or_the_spaces_of_the_site() {
    let jira = world(ConnectionKind::Jira, true, None).await;
    let confluence = world(ConnectionKind::Confluence, true, None).await;

    let projects = jira.service.suggestions(jira.connection_id, Some("en")).await.unwrap();
    let spaces = confluence.service.suggestions(confluence.connection_id, None).await.unwrap();

    assert_eq!(projects.iter().map(|suggestion| (suggestion.value.as_str(), suggestion.name.as_str())).collect::<Vec<_>>(), [("ENG", "ENG project"), ("OPS", "OPS project")]);
    assert_eq!(jira.jira.asked.lock().unwrap().as_slice(), ["en"]);
    assert_eq!(spaces[0].value, "DOCS");
}

#[tokio::test]
async fn a_connection_that_is_neither_has_nothing_to_suggest() {
    let world = world(ConnectionKind::Gitlab, true, None).await;

    assert!(world.service.suggestions(world.connection_id, None).await.unwrap().is_empty());
}
