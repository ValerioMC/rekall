//! Connections shared with companies, the projects that bind them and their access log, against the schema.

use rekall_kernel::{Id, Instant};
use rekall_store::prelude::*;
use rekall_store::{company, connection_access, connection_company, external_connection, project, project_connection, task};
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
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    project
}

async fn an_unshared_cluster(db: &impl ConnectionTrait, label: &str) -> external_connection::Model {
    let connection = external_connection::Model::new(label.into(), label.into(), ConnectionKind::Kubernetes, None, Instant::now());
    connection.clone().into_active_model().insert(db).await.unwrap();
    connection
}

async fn a_cluster(db: &impl ConnectionTrait, company: &company::Model, label: &str) -> external_connection::Model {
    let connection = an_unshared_cluster(db, label).await;
    repo::connection_company::replace_companies(db, connection.id, &[company.id]).await.unwrap();
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
async fn deleting_a_company_unshares_its_connections_and_drops_its_projects_bindings_but_keeps_the_connections() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let globex = a_company(db, "Globex").await;
    let vega = a_project(db, &acme, "vega").await;
    let prod = a_cluster(db, &acme, "prod").await;
    repo::connection_company::replace_companies(db, prod.id, &[acme.id, globex.id]).await.unwrap();
    a_binding(db, &vega, &prod).await;
    repo::connection_access::record(db, connection_access::Model::granted(prod.id, None, "t-1".into(), Instant::now())).await.unwrap();

    Company::delete_by_id(acme.id).exec(db).await.unwrap();

    assert!(repo::external_connection::find_by_id(db, prod.id).await.unwrap().is_some());
    assert_eq!(repo::connection_company::company_ids_of_connection(db, prod.id).await.unwrap(), [globex.id]);
    assert!(ProjectConnection::find().all(db).await.unwrap().is_empty());
    assert_eq!(ConnectionAccess::find().all(db).await.unwrap().len(), 1);
}

#[tokio::test]
async fn deleting_a_connection_takes_its_sharing_its_bindings_and_its_log() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let prod = a_cluster(db, &acme, "prod").await;
    a_binding(db, &vega, &prod).await;
    repo::connection_access::record(db, connection_access::Model::granted(prod.id, None, "t-1".into(), Instant::now())).await.unwrap();

    ExternalConnection::delete_by_id(prod.id).exec(db).await.unwrap();

    assert!(connection_company::Entity::find().all(db).await.unwrap().is_empty());
    assert!(ProjectConnection::find().all(db).await.unwrap().is_empty());
    assert!(ConnectionAccess::find().all(db).await.unwrap().is_empty());
    assert!(Company::find_by_id(acme.id).one(db).await.unwrap().is_some());
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
async fn two_connections_cannot_share_a_label_even_for_different_companies() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let globex = a_company(db, "Globex").await;
    a_cluster(db, &acme, "prod").await;
    let twin = external_connection::Model::new("prod".into(), "Prod".into(), ConnectionKind::Kubernetes, None, Instant::now());

    assert!(twin.into_active_model().insert(db).await.is_err());
    a_cluster(db, &globex, "staging").await;
    assert!(repo::external_connection::label_taken(db, "PROD", None).await.unwrap());
    assert!(!repo::external_connection::label_taken(db, "prod", repo::external_connection::find_by_label(db, "prod").await.unwrap().map(|c| c.id)).await.unwrap());
    assert_eq!(repo::external_connection::find_by_label(db, "Prod").await.unwrap().map(|c| c.label), Some("prod".to_string()));
}

#[tokio::test]
async fn a_company_lists_the_connections_shared_with_it_and_a_connection_the_companies_it_is_shared_with() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let globex = a_company(db, "Globex").await;
    let prod = a_cluster(db, &acme, "prod").await;
    let staging = an_unshared_cluster(db, "staging").await;
    let lab = an_unshared_cluster(db, "lab").await;
    repo::connection_company::replace_companies(db, staging.id, &[globex.id, acme.id]).await.unwrap();

    let of_acme: Vec<String> = repo::external_connection::list_of_company(db, acme.id).await.unwrap().into_iter().map(|c| c.label).collect();
    let sharing_staging: Vec<String> = repo::connection_company::companies_of_connection(db, staging.id).await.unwrap().into_iter().map(|c| c.name).collect();
    let all: Vec<String> = repo::external_connection::list_all(db).await.unwrap().into_iter().map(|c| c.label).collect();

    assert_eq!(of_acme, ["prod", "staging"]);
    assert_eq!(sharing_staging, ["Acme", "Globex"]);
    assert_eq!(all, ["lab", "prod", "staging"]);
    assert!(repo::connection_company::companies_of_connection(db, lab.id).await.unwrap().is_empty());
    assert!(repo::external_connection::list_of_company(db, globex.id).await.unwrap().iter().all(|c| c.id != prod.id));
}

#[tokio::test]
async fn replacing_the_companies_of_a_connection_leaves_exactly_those_and_counts_a_repeated_one_once() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let globex = a_company(db, "Globex").await;
    let prod = a_cluster(db, &acme, "prod").await;

    repo::connection_company::replace_companies(db, prod.id, &[globex.id, globex.id]).await.unwrap();

    assert!(!repo::connection_company::is_associated(db, prod.id, acme.id).await.unwrap());
    assert!(repo::connection_company::is_associated(db, prod.id, globex.id).await.unwrap());
    assert_eq!(repo::connection_company::company_ids_of_connection(db, prod.id).await.unwrap(), [globex.id]);

    repo::connection_company::replace_companies(db, prod.id, &[]).await.unwrap();
    assert!(repo::connection_company::company_ids_of_connection(db, prod.id).await.unwrap().is_empty());
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
