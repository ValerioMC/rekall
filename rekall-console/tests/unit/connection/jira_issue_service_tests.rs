use std::sync::Mutex;

use async_trait::async_trait;
use rekall_common::Instant;
use rekall_store::{company, external_connection, project, project_connection, ProjectStatus};
use rekall_service::connection::{InMemorySecretStore, Secret};
use rekall_service::{EventBus, Services};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

/// Answers every search with the issues it holds, and remembers the queries it was asked.
struct FakeJira {
    issues: Vec<JiraIssue>,
    failure: Option<JiraFailure>,
    asked: Mutex<Vec<String>>,
}

#[async_trait]
impl JiraIssues for FakeJira {
    async fn search(&self, _site: &JiraSite, credential: &JiraCredential, jql: &str, _limit: u16) -> std::result::Result<Vec<JiraIssue>, JiraFailure> {
        assert_eq!(credential.email(), Some("ada@example.com"));
        self.asked.lock().unwrap().push(jql.to_string());
        match &self.failure {
            Some(failure) => Err(failure.clone()),
            None => Ok(self.issues.clone()),
        }
    }
}

struct World {
    service: JiraIssueService,
    jira: Arc<FakeJira>,
    project_id: Id,
    binding_id: Id,
}

fn issue(key: &str, summary: &str) -> JiraIssue {
    JiraIssue { key: key.into(), summary: summary.into(), status: Some("To Do".into()), issue_type: Some("Story".into()), description: None }
}

async fn world(kind: ConnectionKind, target: Option<&str>, failure: Option<JiraFailure>) -> World {
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
        auto_commit: false,
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let mut connection = external_connection::Model::new(company.id, "jira".into(), "Jira".into(), kind, Some("https://example.atlassian.net".into()), now);
    connection.secret_set_at = Some(now);
    connection.clone().into_active_model().insert(&database.conn).await.unwrap();
    let binding = project_connection::Model::new(project.id, connection.id, connection.label.clone(), target.map(str::to_string), now);
    binding.clone().into_active_model().insert(&database.conn).await.unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());
    secrets.put(&connection.secret_ref, &Secret::new("ada@example.com:atl-secret")).unwrap();
    let services = Services::new(ctx.clone());
    let tasks = TaskService::new(ctx.clone(), services.review.clone(), services.time_entries.clone());
    let jira = Arc::new(FakeJira { issues: vec![issue("ENG-1", "Fix login"), issue("ENG-2", "Add export")], failure, asked: Mutex::new(Vec::new()) });
    let service = JiraIssueService::new(ctx.clone(), ProjectConnectionService::new(ctx), tasks, secrets, jira.clone());
    World { service, jira, project_id: project.id, binding_id: binding.id }
}

fn keys(keys: &[&str]) -> JiraImportCommand {
    JiraImportCommand { keys: keys.iter().map(|key| key.to_string()).collect() }
}

#[tokio::test]
async fn the_open_issues_of_the_bound_project_are_listed_with_a_link_and_no_task_yet() {
    let world = world(ConnectionKind::Jira, Some("eng"), None).await;

    let listed = world.service.list(world.project_id, world.binding_id).await.unwrap();

    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].url, "https://example.atlassian.net/browse/ENG-1");
    assert!(listed.iter().all(|issue| issue.task_id.is_none()));
    assert!(world.jira.asked.lock().unwrap()[0].starts_with("project = \"ENG\""));
}

#[tokio::test]
async fn importing_creates_one_task_per_issue_and_a_second_import_adds_nothing() {
    let world = world(ConnectionKind::Jira, Some("ENG"), None).await;

    let created = world.service.import(world.project_id, world.binding_id, keys(&["ENG-1", "ENG-2"])).await.unwrap();
    let again = world.service.import(world.project_id, world.binding_id, keys(&["ENG-1"])).await.unwrap();
    let listed = world.service.list(world.project_id, world.binding_id).await.unwrap();

    assert_eq!(created.iter().map(|task| task.label.as_str()).collect::<Vec<_>>(), ["eng-1", "eng-2"]);
    assert_eq!(created[0].title, "Fix login");
    assert!(again.is_empty());
    assert_eq!(listed[0].task_id, Some(created[0].id));
}

#[tokio::test]
async fn a_binding_without_a_project_key_or_of_another_kind_is_refused_before_jira_is_asked() {
    let without_key = world(ConnectionKind::Jira, None, None).await;
    assert!(without_key.service.list(without_key.project_id, without_key.binding_id).await.unwrap_err().is_illegal_argument());

    let gitlab = world(ConnectionKind::Gitlab, Some("ENG"), None).await;
    assert!(gitlab.service.list(gitlab.project_id, gitlab.binding_id).await.unwrap_err().is_illegal_argument());
    assert!(gitlab.jira.asked.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_credential_jira_rejects_is_answered_as_a_conflict_and_nothing_is_created() {
    let world = world(ConnectionKind::Jira, Some("ENG"), Some(JiraFailure::Rejected { status: 401 })).await;

    let error = world.service.import(world.project_id, world.binding_id, keys(&["ENG-1"])).await.unwrap_err();

    assert!(matches!(error, RekallError::Conflict(_)));
    assert!(error.message().contains("HTTP 401"));
}
