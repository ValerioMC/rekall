//! The migrations, run the two ways a database meets them: from nothing, and from a file that
//! already held data under an older schema.

use rekall_common::{Id, Instant};
use rekall_model::prelude::*;
use rekall_model::{company, document, project, task, task_step, wrapup};
use rekall_repository::migration::Migrator;
use sea_orm::sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, IntoActiveModel, SqlxSqliteConnector};
use sea_orm_migration::MigratorTrait;

#[tokio::test]
async fn a_fresh_schema_holds_every_entity() {
    let database = rekall_repository::open_temporary().await.unwrap();
    let db = &database.conn;
    let now = Instant::now();

    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(),
        label: "vega".into(),
        title: "Vega".into(),
        status: ProjectStatus::Active,
        description: None,
        blueprint_markdown: None,
        repo_folder: None,
        auto_commit: false,
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    let task = task::Model::new("report".into(), "Report".into(), project.id, now);
    task.clone().into_active_model().insert(db).await.unwrap();
    let step = task_step::Model::new(task.id, "First".into(), 0, now);
    step.clone().into_active_model().insert(db).await.unwrap();
    wrapup::Model { id: Id::random(), task_id: task.id, body_markdown: "state".into(), written_by: WrapupAuthor::Claude, created_at: now, updated_at: now }
        .into_active_model()
        .insert(db)
        .await
        .unwrap();

    let read = Task::find_by_id(task.id).one(db).await.unwrap().unwrap();
    assert_eq!(read, task);
    let read_step = TaskStep::find_by_id(step.id).one(db).await.unwrap().unwrap();
    assert_eq!(read_step.state, TaskStepState::Draft);

    // Deleting the company takes everything under it: the cascades survived every rebuild.
    Company::delete_by_id(company.id).exec(db).await.unwrap();
    assert_eq!(Task::find().all(db).await.unwrap().len(), 0);
    assert_eq!(TaskStep::find().all(db).await.unwrap().len(), 0);
    assert_eq!(Wrapup::find().all(db).await.unwrap().len(), 0);
}

#[tokio::test]
async fn a_second_task_with_the_same_label_on_a_project_is_refused_by_name() {
    let database = rekall_repository::open_temporary().await.unwrap();
    let db = &database.conn;
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active,
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false,
        company_id: company.id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    task::Model::new("report".into(), "Report".into(), project.id, now).into_active_model().insert(db).await.unwrap();
    let error = task::Model::new("report".into(), "Again".into(), project.id, now)
        .into_active_model()
        .insert(db)
        .await
        .unwrap_err();
    let mapped = rekall_common::RekallError::from(error);
    assert!(mapped.message().contains("UQ_TASK_PROJECT_LABEL"), "{mapped:?}");
}

#[tokio::test]
async fn a_database_from_before_companies_and_labels_comes_through_whole() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("rekall.db");
    let options = SqliteConnectOptions::new().filename(&path).create_if_missing(true).foreign_keys(false);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    let conn = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);

    // The schema as 001-core.yaml left it, with data in it.
    Migrator::up(&conn, Some(4)).await.unwrap();
    let now = "2024-01-01T00:00:00.000000Z";
    for statement in [
        format!("INSERT INTO project VALUES ('11111111-1111-1111-1111-111111111111', 'Progetto Vega', 'ACTIVE', NULL, '{now}', '{now}')"),
        format!("INSERT INTO task VALUES ('22222222-2222-2222-2222-222222222222', 'Report Builder!', 'TODO', NULL, '11111111-1111-1111-1111-111111111111', NULL, '{now}', '{now}')"),
        format!("INSERT INTO document VALUES ('33333333-3333-3333-3333-333333333333', 'Cluster', 'context', 'body', NULL, 0, NULL, '22222222-2222-2222-2222-222222222222', NULL, '{now}', '{now}')"),
        format!("INSERT INTO document VALUES ('44444444-4444-4444-4444-444444444444', 'Owned by the project', 'context', 'body', NULL, 0, '11111111-1111-1111-1111-111111111111', NULL, NULL, '{now}', '{now}')"),
    ] {
        conn.execute_unprepared(&statement).await.unwrap();
    }
    Migrator::up(&conn, None).await.unwrap();
    conn.close().await.unwrap();

    let database = rekall_repository::open(&path).await.unwrap();
    let db = &database.conn;

    let companies = Company::find().all(db).await.unwrap();
    assert_eq!(companies.len(), 1, "one holding company for the projects that were there");
    assert_eq!(companies[0].name, "Unassigned");

    let project = Project::find().one(db).await.unwrap().unwrap();
    assert_eq!(project.label, "progetto-vega");
    assert_eq!(project.title, "Progetto Vega");
    assert_eq!(project.company_id, companies[0].id);

    let task = Task::find().one(db).await.unwrap().unwrap();
    assert_eq!(task.label, "report-builder");
    assert_eq!(task.title, "Report Builder!");
    assert_eq!(task.review_state, TaskStepState::Open);

    let documents = Document::find().all(db).await.unwrap();
    assert_eq!(documents.len(), 1, "a note with no task is dropped, a task's note is kept");
    assert_eq!(documents[0].context_mode, DocumentContextMode::Full);
    let links = DocumentTask::find().all(db).await.unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].task_id, task.id);

    let tables: Vec<String> = db
        .query_all_raw(sea_orm::Statement::from_string(
            db.get_database_backend(),
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        ))
        .await
        .unwrap()
        .iter()
        .map(|row| row.try_get_by_index::<String>(0).unwrap())
        .collect();
    assert_eq!(
        tables,
        [
            "commit_reference", "company", "document", "document_task", "project", "run_queue",
            "run_queue_item", "seaql_migrations", "tag", "task", "task_revision", "task_step",
            "time_entry", "wrapup"
        ]
    );
    let _ = document::ANCHOR_ID_LENGTH;
}
