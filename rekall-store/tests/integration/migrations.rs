//! The migrations, run the ways a database meets them: from nothing, from a file a v0.2.x
//! release wrote, and from a file too old to carry forward.

use rekall_kernel::{Id, Instant};
use rekall_store::prelude::*;
use rekall_store::{company, project, task, task_step, wrapup};
use sea_orm::sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, IntoActiveModel, SqlxSqliteConnector};

#[tokio::test]
async fn a_fresh_schema_holds_every_entity() {
    let database = rekall_store::open_temporary().await.unwrap();
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
    let database = rekall_store::open_temporary().await.unwrap();
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
    let mapped = rekall_kernel::RekallError::from(error);
    assert!(mapped.message().contains("task.project_id, task.label"), "{mapped:?}");
}

/// A SQLite file at `path` holding what `script` writes, opened the way an old release left it.
async fn a_file_from(path: &std::path::Path, script: &'static str) {
    let options = SqliteConnectOptions::new().filename(path).create_if_missing(true).foreign_keys(false);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    sea_orm::sqlx::raw_sql(script).execute(&pool).await.unwrap();
    pool.close().await;
}

async fn table_exists(db: &impl ConnectionTrait, name: &str) -> bool {
    db.query_one_raw(sea_orm::Statement::from_sql_and_values(
        db.get_database_backend(),
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?",
        [name.into()],
    ))
    .await
    .unwrap()
    .is_some()
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

    let database = rekall_store::open(&path).await.unwrap();
    let db = &database.conn;

    assert_eq!(applied_versions(db).await, ["000-baseline", "031-project-section", "032-project-icon-image", "033-project-component", "034-company-logo-image", "035-mermaid-diagram", "036-token-usage", "037-skill-placement", "038-skill-assignment", "039-step-flow", "040-flow-start-end", "041-token-usage-day", "042-step-max-visits", "043-diagram-walkthrough", "044-connection", "045-connection-proxy", "046-drop-task-revision", "047-vulnerability", "048-exposure-conclusion", "049-deployment-graph", "050-graph-step-place", "051-binding-label", "052-advisory-reach", "053-visit-report", "054-release-on-run", "055-epic", "056-step-session", "057-telemetry-setting", "058-run-queue-session-kind", "059-shared-connection", "060-step-snapshot", "061-done-check", "062-wrapup-version", "063-quick-task", "064-task-dependency", "065-parallel-sessions", "066-task-worktree", "067-task-deviation"]);
    let task = Task::find().one(db).await.unwrap().unwrap();
    assert_eq!((task.label.as_str(), task.title.as_str()), ("report", "Report"));
    let wrapup = Wrapup::find().one(db).await.unwrap().unwrap();
    assert_eq!(wrapup.body_markdown, "The report renders.");
    assert_eq!(wrapup.task_id, task.id);
    let versions = rekall_store::wrapup::wrapup_version::Entity::find().all(db).await.unwrap();
    assert_eq!(versions.len(), 1, "an existing wrapup starts with its text as its first version");
    assert_eq!((versions[0].number, versions[0].body_markdown.as_str()), (1, "The report renders."));
    assert!(!table_exists(db, "task_revision").await, "earlier versions of a text are no longer kept");
}

#[tokio::test]
async fn a_database_on_the_baseline_opens_again_unchanged() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("rekall.db");
    rekall_store::open(&path).await.unwrap().close().await.unwrap();

    let database = rekall_store::open(&path).await.unwrap();

    assert_eq!(applied_versions(&database.conn).await, ["000-baseline", "031-project-section", "032-project-icon-image", "033-project-component", "034-company-logo-image", "035-mermaid-diagram", "036-token-usage", "037-skill-placement", "038-skill-assignment", "039-step-flow", "040-flow-start-end", "041-token-usage-day", "042-step-max-visits", "043-diagram-walkthrough", "044-connection", "045-connection-proxy", "046-drop-task-revision", "047-vulnerability", "048-exposure-conclusion", "049-deployment-graph", "050-graph-step-place", "051-binding-label", "052-advisory-reach", "053-visit-report", "054-release-on-run", "055-epic", "056-step-session", "057-telemetry-setting", "058-run-queue-session-kind", "059-shared-connection", "060-step-snapshot", "061-done-check", "062-wrapup-version", "063-quick-task", "064-task-dependency", "065-parallel-sessions", "066-task-worktree", "067-task-deviation"]);
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

    let refused = rekall_store::open(&path).await.unwrap_err().to_string();

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

    assert!(rekall_store::check_openable(&old).await.unwrap_err().to_string().contains("v0.2.2"));
    rekall_store::check_openable(&current).await.unwrap();

    let options = SqliteConnectOptions::new().filename(&current);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    let conn = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    assert_eq!(applied_versions(&conn).await.len(), 41, "a check writes nothing");
}

#[tokio::test]
async fn a_binding_made_before_051_takes_its_connections_label_and_a_project_may_then_bind_it_twice() {
    use sea_orm_migration::MigratorTrait;
    let options = SqliteConnectOptions::new().filename(":memory:").foreign_keys(false);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    let db = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    let before_051 = rekall_store::migration::Migrator::migrations().iter().position(|migration| migration.name() == "051-binding-label").unwrap() as u32;
    rekall_store::migration::Migrator::up(&db, Some(before_051)).await.unwrap();
    for statement in [
        "INSERT INTO company (id, name, created_at, updated_at) VALUES ('c', 'Acme', 't', 't')",
        "INSERT INTO project (id, label, title, status, icon, auto_commit, company_id, created_at, updated_at) VALUES ('p', 'vega', 'Vega', 'ACTIVE', 'folder', 0, 'c', 't', 't')",
        "INSERT INTO external_connection (id, company_id, label, title, kind, base_url, secret_ref, created_at, updated_at) VALUES ('g', 'c', 'gitlab', 'GitLab', 'GITLAB', 'https://gitlab.example.com', 'connection-g', 't', 't')",
        "INSERT INTO project_connection (id, project_id, connection_id, target, created_at, updated_at) VALUES ('b', 'p', 'g', 'acme/vega', 't', 't')",
    ] {
        db.execute_unprepared(statement).await.unwrap();
    }

    rekall_store::migration::Migrator::up(&db, None).await.unwrap();

    let label = db
        .query_one_raw(sea_orm::Statement::from_string(db.get_database_backend(), "SELECT label FROM project_connection WHERE id = 'b'"))
        .await
        .unwrap()
        .unwrap()
        .try_get_by_index::<String>(0)
        .unwrap();
    assert_eq!(label, "gitlab");
    db.execute_unprepared("INSERT INTO project_connection (id, project_id, connection_id, label, target, created_at, updated_at) VALUES ('b2', 'p', 'g', 'billing', 'acme/billing', 't', 't')")
        .await
        .unwrap();
    assert!(db
        .execute_unprepared("INSERT INTO project_connection (id, project_id, connection_id, label, created_at, updated_at) VALUES ('b3', 'p', 'g', 'billing', 't', 't')")
        .await
        .is_err());
}

#[tokio::test]
async fn an_exposure_judged_before_052_keeps_its_verdict_and_a_second_version_of_its_package_may_join_it() {
    use sea_orm_migration::MigratorTrait;
    let options = SqliteConnectOptions::new().filename(":memory:").foreign_keys(false);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    let db = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    let before_052 = rekall_store::migration::Migrator::migrations().iter().position(|migration| migration.name() == "052-advisory-reach").unwrap() as u32;
    rekall_store::migration::Migrator::up(&db, Some(before_052)).await.unwrap();
    for statement in [
        "INSERT INTO company (id, name, created_at, updated_at) VALUES ('c', 'Acme', 't', 't')",
        "INSERT INTO project (id, label, title, status, icon, auto_commit, company_id, created_at, updated_at) VALUES ('p', 'vega', 'Vega', 'ACTIVE', 'folder', 0, 'c', 't', 't')",
        "INSERT INTO security_advisory (id, ghsa_id, summary, description, url, published_at, affected_json, created_at, updated_at) VALUES ('a', 'GHSA-aaaa-aaaa-aaaa', 'RCE', '', 'u', 't', '[]', 't', 't')",
        "INSERT INTO vulnerability_exposure (id, project_id, advisory_id, ecosystem, package_name, package_version, verdict, reasoning, conclusion, assessed_at, created_at, updated_at) VALUES ('e', 'p', 'a', 'RUST', 'hashbrown', '0.12.3', 'NOT_AFFECTED', 'never called', 'Vega never hashes input.', 't', 't', 't')",
    ] {
        db.execute_unprepared(statement).await.unwrap();
    }

    rekall_store::migration::Migrator::up(&db, None).await.unwrap();

    let kept = db
        .query_one_raw(sea_orm::Statement::from_string(db.get_database_backend(), "SELECT verdict || '|' || conclusion || '|' || severity FROM vulnerability_exposure JOIN security_advisory ON security_advisory.id = advisory_id WHERE vulnerability_exposure.id = 'e'"))
        .await
        .unwrap()
        .unwrap()
        .try_get_by_index::<String>(0)
        .unwrap();
    assert_eq!(kept, "NOT_AFFECTED|Vega never hashes input.|CRITICAL");
    db.execute_unprepared("INSERT INTO vulnerability_exposure (id, project_id, advisory_id, ecosystem, package_name, package_version, created_at, updated_at) VALUES ('e2', 'p', 'a', 'RUST', 'hashbrown', '0.14.5', 't', 't')")
        .await
        .unwrap();
    assert!(db
        .execute_unprepared("INSERT INTO vulnerability_exposure (id, project_id, advisory_id, ecosystem, package_name, package_version, created_at, updated_at) VALUES ('e3', 'p', 'a', 'RUST', 'hashbrown', '0.14.5', 't', 't')")
        .await
        .is_err());
}

async fn text_rows(db: &impl ConnectionTrait, sql: &str) -> Vec<String> {
    db.query_all_raw(sea_orm::Statement::from_string(db.get_database_backend(), sql))
        .await
        .unwrap()
        .iter()
        .map(|row| row.try_get_by_index::<String>(0).unwrap())
        .collect()
}

#[tokio::test]
async fn a_connection_owned_by_a_company_before_059_is_shared_with_it_and_a_repeated_label_is_renamed() {
    use sea_orm_migration::MigratorTrait;
    let options = SqliteConnectOptions::new().filename(":memory:").foreign_keys(false);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    let db = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    let before_059 = rekall_store::migration::Migrator::migrations().iter().position(|migration| migration.name() == "059-shared-connection").unwrap() as u32;
    rekall_store::migration::Migrator::up(&db, Some(before_059)).await.unwrap();
    for statement in [
        "INSERT INTO company (id, name, created_at, updated_at) VALUES ('acme', 'Acme', 't', 't'), ('globex', 'Globex', 't', 't')",
        "INSERT INTO project (id, label, title, status, icon, auto_commit, company_id, created_at, updated_at) VALUES ('p', 'vega', 'Vega', 'ACTIVE', 'folder', 0, 'acme', 't', 't')",
        "INSERT INTO external_connection (id, company_id, label, title, kind, base_url, secret_ref, created_at, updated_at) VALUES
            ('acme-gitlab', 'acme', 'gitlab', 'GitLab', 'GITLAB', 'https://gitlab.example.com', 'connection-1', '2024-01-01', 't'),
            ('globex-gitlab', 'globex', 'gitlab', 'GitLab', 'GITLAB', 'https://gitlab.example.com', 'connection-2', '2024-02-01', 't'),
            ('globex-taken', 'globex', 'gitlab-2', 'Other', 'GITLAB', 'https://gitlab.example.com', 'connection-3', '2024-03-01', 't')",
        "INSERT INTO project_connection (id, project_id, connection_id, label, target, created_at, updated_at) VALUES ('b', 'p', 'acme-gitlab', 'gitlab', 'acme/vega', 't', 't')",
    ] {
        db.execute_unprepared(statement).await.unwrap();
    }

    rekall_store::migration::Migrator::up(&db, None).await.unwrap();

    assert_eq!(
        text_rows(&db, "SELECT id || ':' || label FROM external_connection ORDER BY id").await,
        ["acme-gitlab:gitlab", "globex-gitlab:gitlab-3", "globex-taken:gitlab-2"]
    );
    assert_eq!(
        text_rows(&db, "SELECT connection_id || '>' || company_id FROM connection_company ORDER BY connection_id").await,
        ["acme-gitlab>acme", "globex-gitlab>globex", "globex-taken>globex"]
    );
    assert!(db.execute_unprepared("SELECT company_id FROM external_connection").await.is_err(), "the connection no longer carries a company");
    assert_eq!(text_rows(&db, "SELECT connection_id FROM project_connection").await, ["acme-gitlab"]);
    assert!(db
        .execute_unprepared("INSERT INTO external_connection (id, label, title, kind, secret_ref, created_at, updated_at) VALUES ('x', 'gitlab', 'Again', 'KUBERNETES', 'connection-4', 't', 't')")
        .await
        .is_err());
}

#[tokio::test]
async fn a_changes_note_on_one_task_becomes_that_tasks_deviation_and_leaves_the_notes() {
    use sea_orm_migration::MigratorTrait;
    let options = SqliteConnectOptions::new().filename(":memory:").foreign_keys(false);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    let db = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
    let before_067 = rekall_store::migration::Migrator::migrations().iter().position(|migration| migration.name() == "067-task-deviation").unwrap() as u32;
    rekall_store::migration::Migrator::up(&db, Some(before_067)).await.unwrap();
    let note = |id: &str, title: &str| {
        format!("INSERT INTO document (id, title, kind, body_markdown, context_mode, created_at, updated_at) VALUES ('{id}', '{title}', 'notes', 'body of {id}', 'FULL', '2026-10-01T09:00:00.000000Z', '2026-10-01T09:00:00.000000Z')")
    };
    for statement in [
        "INSERT INTO company (id, name, created_at, updated_at) VALUES ('c', 'Acme', 't', 't')".to_string(),
        "INSERT INTO project (id, label, title, status, icon, auto_commit, company_id, created_at, updated_at) VALUES ('p', 'vega', 'Vega', 'ACTIVE', 'folder', 0, 'c', 't', 't')".to_string(),
        "INSERT INTO task (id, label, title, status, project_id, created_at, updated_at) VALUES ('00000000-0000-0000-0000-0000000000a1', 'one', 'One', 'TODO', 'p', 't', 't')".to_string(),
        "INSERT INTO task (id, label, title, status, project_id, created_at, updated_at) VALUES ('00000000-0000-0000-0000-0000000000a2', 'two', 'Two', 'TODO', 'p', 't', 't')".to_string(),
        note("00000000-0000-0000-0000-0000000000b3", "Step-3-Changes.md"),
        note("00000000-0000-0000-0000-0000000000b4", "step-4-changes.md"),
        note("00000000-0000-0000-0000-0000000000b5", "step-x-changes.md"),
        note("00000000-0000-0000-0000-0000000000b6", "analysis.md"),
        "INSERT INTO document_task (document_id, task_id, position) VALUES ('00000000-0000-0000-0000-0000000000b3', '00000000-0000-0000-0000-0000000000a1', 0), ('00000000-0000-0000-0000-0000000000b4', '00000000-0000-0000-0000-0000000000a1', 1), ('00000000-0000-0000-0000-0000000000b4', '00000000-0000-0000-0000-0000000000a2', 0), ('00000000-0000-0000-0000-0000000000b5', '00000000-0000-0000-0000-0000000000a1', 2), ('00000000-0000-0000-0000-0000000000b6', '00000000-0000-0000-0000-0000000000a1', 3)".to_string(),
    ] {
        db.execute_unprepared(&statement).await.unwrap();
    }

    rekall_store::migration::Migrator::up(&db, None).await.unwrap();

    let deviations = TaskDeviation::find().all(&db).await.unwrap();
    assert_eq!(deviations.len(), 1, "only a note on one task moves");
    assert_eq!((deviations[0].step_number, deviations[0].body_markdown.as_str()), (3, "body of 00000000-0000-0000-0000-0000000000b3"));
    let mut notes: Vec<String> = Document::find().all(&db).await.unwrap().into_iter().map(|note| note.title).collect();
    notes.sort();
    assert_eq!(notes, ["analysis.md", "step-4-changes.md", "step-x-changes.md"]);
}
