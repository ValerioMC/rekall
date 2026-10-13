//! The migrations, run the ways a database meets them: from nothing, from a file a v0.2.x
//! release wrote, and from a file too old to carry forward.

use rekall_common::{Id, Instant};
use rekall_model::prelude::*;
use rekall_model::{company, project, task, task_step, wrapup};
use sea_orm::sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, IntoActiveModel, SqlxSqliteConnector};

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
        icon: "folder".into(),
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

    // Deleting the company takes everything under it.
    Company::delete_by_id(company.id).exec(db).await.unwrap();
    assert_eq!(Task::find().all(db).await.unwrap().len(), 0);
    assert_eq!(TaskStep::find().all(db).await.unwrap().len(), 0);
    assert_eq!(Wrapup::find().all(db).await.unwrap().len(), 0);
}

#[tokio::test]
async fn a_second_task_with_the_same_label_on_a_project_is_refused_by_the_schema_too() {
    let database = rekall_repository::open_temporary().await.unwrap();
    let db = &database.conn;
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active,
        icon: "folder".into(), description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false,
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
    assert!(mapped.message().contains("task.project_id, task.label"), "{mapped:?}");
}

/// A SQLite file at `path` holding what `script` writes, opened the way an old release left it.
async fn a_file_from(path: &std::path::Path, script: &'static str) {
    let options = SqliteConnectOptions::new().filename(path).create_if_missing(true).foreign_keys(false);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    sea_orm::sqlx::raw_sql(script).execute(&pool).await.unwrap();
    pool.close().await;
}

async fn applied_versions(db: &impl ConnectionTrait) -> Vec<String> {
    db.query_all_raw(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "SELECT version FROM seaql_migrations ORDER BY version",
    ))
    .await
    .unwrap()
    .iter()
    .map(|row| row.try_get_by_index::<String>(0).unwrap())
    .collect()
}

#[tokio::test]
async fn a_database_written_by_v0_2_moves_onto_the_baseline_with_its_rows() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("rekall.db");
    a_file_from(&path, include_str!("fixtures/v0.2-database.sql")).await;

    let database = rekall_repository::open(&path).await.unwrap();
    let db = &database.conn;

    assert_eq!(applied_versions(db).await, ["000-baseline", "031-project-section", "032-project-icon-image", "033-project-component", "034-company-logo-image", "035-mermaid-diagram", "036-token-usage", "037-skill-placement", "038-skill-assignment", "039-step-flow"]);
    let task = Task::find().one(db).await.unwrap().unwrap();
    assert_eq!((task.label.as_str(), task.title.as_str()), ("report", "Report"));
    let wrapup = Wrapup::find().one(db).await.unwrap().unwrap();
    assert_eq!(wrapup.body_markdown, "The report renders.");
    assert_eq!(wrapup.task_id, task.id);
}

#[tokio::test]
async fn a_database_on_the_baseline_opens_again_unchanged() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("rekall.db");
    rekall_repository::open(&path).await.unwrap().close().await.unwrap();

    let database = rekall_repository::open(&path).await.unwrap();

    assert_eq!(applied_versions(&database.conn).await, ["000-baseline", "031-project-section", "032-project-icon-image", "033-project-component", "034-company-logo-image", "035-mermaid-diagram", "036-token-usage", "037-skill-placement", "038-skill-assignment", "039-step-flow"]);
}

#[tokio::test]
async fn a_database_older_than_v0_2_is_refused_and_left_as_it_was() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("rekall.db");
    a_file_from(
        &path,
        "CREATE TABLE seaql_migrations (version VARCHAR NOT NULL PRIMARY KEY, applied_at BIGINT NOT NULL);
         INSERT INTO seaql_migrations VALUES ('001-core', 0), ('028-diagram', 0);",
    )
    .await;

    let refused = rekall_repository::open(&path).await.unwrap_err().to_string();

    assert!(refused.contains("older than v0.2.0") && refused.contains("v0.2.2"), "{refused}");
    let options = SqliteConnectOptions::new().filename(&path);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    let conn = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    assert_eq!(applied_versions(&conn).await, ["001-core", "028-diagram"]);
}

#[tokio::test]
async fn checking_a_file_before_a_restore_refuses_one_too_old_without_touching_it() {
    let folder = tempfile::tempdir().unwrap();
    let old = folder.path().join("old.db");
    a_file_from(&old, "CREATE TABLE seaql_migrations (version VARCHAR NOT NULL PRIMARY KEY, applied_at BIGINT NOT NULL);
                       INSERT INTO seaql_migrations VALUES ('028-diagram', 0);").await;
    let current = folder.path().join("current.db");
    a_file_from(&current, include_str!("fixtures/v0.2-database.sql")).await;

    assert!(rekall_repository::check_openable(&old).await.unwrap_err().to_string().contains("v0.2.2"));
    rekall_repository::check_openable(&current).await.unwrap();

    let options = SqliteConnectOptions::new().filename(&current);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    let conn = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    assert_eq!(applied_versions(&conn).await.len(), 41, "a check writes nothing");
}
