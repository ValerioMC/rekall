//! A company's connections, the projects that bind them and their access log, against the schema.

use rekall_common::{Id, Instant};
use rekall_store::prelude::*;
use rekall_store::{company, connection_access, external_connection, project, project_connection, task};
use rekall_store::repository as repo;
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, IntoActiveModel};

async fn a_company(db: &impl ConnectionTrait, name: &str) -> company::Model {
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: name.into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    company
}

async fn a_project(db: &impl ConnectionTrait, company: &company::Model, label: &str) -> project::Model {
    let now = Instant::now();
    let project = project::Model {
        id: Id::random(), label: label.into(), title: label.into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    project
}

async fn a_cluster(db: &impl ConnectionTrait, company: &company::Model, label: &str) -> external_connection::Model {
    let connection = external_connection::Model::new(company.id, label.into(), label.into(), ConnectionKind::Kubernetes, None, Instant::now());
    connection.clone().into_active_model().insert(db).await.unwrap();
    connection
}

async fn a_binding(db: &impl ConnectionTrait, project: &project::Model, connection: &external_connection::Model) -> project_connection::Model {
    a_labelled_binding(db, project, connection, &connection.label).await
}

async fn a_labelled_binding(db: &impl ConnectionTrait, project: &project::Model, connection: &external_connection::Model, label: &str) -> project_connection::Model {
    repo::project_connection::bind(db, project_connection::Model::new(project.id, connection.id, label.into(), Some("apps".into()), Instant::now()))
        .await
        .unwrap()
}

#[tokio::test]
async fn deleting_a_company_takes_its_connections_their_bindings_and_their_log() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let prod = a_cluster(db, &acme, "prod").await;
    a_binding(db, &vega, &prod).await;
    repo::connection_access::record(db, connection_access::Model::granted(prod.id, None, "t-1".into(), Instant::now())).await.unwrap();

    Company::delete_by_id(acme.id).exec(db).await.unwrap();

    assert!(ExternalConnection::find().all(db).await.unwrap().is_empty());
    assert!(ProjectConnection::find().all(db).await.unwrap().is_empty());
    assert!(ConnectionAccess::find().all(db).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_project_binds_a_connection_once_per_label() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let prod = a_cluster(db, &acme, "prod").await;
    a_binding(db, &vega, &prod).await;
    a_labelled_binding(db, &vega, &prod, "billing").await;

    let again = repo::project_connection::bind(db, project_connection::Model::new(vega.id, prod.id, "billing".into(), None, Instant::now())).await;

    assert!(again.is_err());
    assert_eq!(repo::project_connection::list_of_project_connection(db, vega.id, prod.id).await.unwrap().len(), 2);
    assert!(repo::project_connection::is_bound(db, vega.id, prod.id).await.unwrap());
}

#[tokio::test]
async fn two_connections_of_a_company_cannot_share_a_label() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    a_cluster(db, &acme, "prod").await;
    let twin = external_connection::Model::new(acme.id, "prod".into(), "Prod".into(), ConnectionKind::Kubernetes, None, Instant::now());

    assert!(twin.into_active_model().insert(db).await.is_err());
    assert!(repo::external_connection::label_taken(db, acme.id, "PROD", None).await.unwrap());
    assert_eq!(repo::external_connection::find_by_label(db, acme.id, "Prod").await.unwrap().map(|c| c.label), Some("prod".to_string()));
}

#[tokio::test]
async fn a_project_lists_its_connections_by_label_and_a_connection_lists_its_projects() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let atlas = a_project(db, &acme, "atlas").await;
    let staging = a_cluster(db, &acme, "staging").await;
    let prod = a_cluster(db, &acme, "prod").await;
    a_binding(db, &vega, &staging).await;
    a_binding(db, &vega, &prod).await;
    a_binding(db, &atlas, &prod).await;

    let of_vega: Vec<String> = repo::project_connection::list_of_project(db, vega.id).await.unwrap().into_iter().map(|(_, c)| c.label).collect();
    let using_prod: Vec<String> = repo::project_connection::list_of_connection(db, prod.id).await.unwrap().into_iter().map(|(_, p)| p.label).collect();

    assert_eq!(of_vega, ["prod", "staging"]);
    assert_eq!(using_prod, ["atlas", "vega"]);
}

#[tokio::test]
async fn unbinding_removes_only_that_binding() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let prod = a_cluster(db, &acme, "prod").await;
    let binding = a_binding(db, &vega, &prod).await;
    let kept = a_labelled_binding(db, &vega, &prod, "billing").await;

    assert!(repo::project_connection::unbind(db, vega.id, binding.id).await.unwrap());
    assert!(!repo::project_connection::unbind(db, vega.id, binding.id).await.unwrap());
    assert!(repo::project_connection::find(db, vega.id, kept.id).await.unwrap().is_some());
    assert!(repo::external_connection::find_by_id(db, prod.id).await.unwrap().is_some());
}

#[tokio::test]
async fn the_access_log_outlives_a_deleted_task_and_reads_newest_first() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let prod = a_cluster(db, &acme, "prod").await;
    let deploy = task::Model::new("deploy".into(), "Deploy".into(), vega.id, Instant::now());
    deploy.clone().into_active_model().insert(db).await.unwrap();
    let first = Instant::now();
    repo::connection_access::record(db, connection_access::Model::granted(prod.id, Some(deploy.id), "t-1".into(), first)).await.unwrap();
    repo::connection_access::record(db, connection_access::Model::refused(prod.id, Some(deploy.id), "t-1".into(), "wrong token".into(), first.plus_seconds(5)))
        .await
        .unwrap();

    Task::delete_by_id(deploy.id).exec(db).await.unwrap();

    let log = repo::connection_access::list_of_connection(db, prod.id, 0, 10).await.unwrap();
    assert_eq!(log.iter().map(|a| a.granted).collect::<Vec<_>>(), [false, true]);
    assert!(log.iter().all(|a| a.task_id.is_none()));
    assert_eq!(repo::connection_access::list_of_connection(db, prod.id, 1, 10).await.unwrap().len(), 1);
}
