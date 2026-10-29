//! `update_if_changed`: the one write path every service uses to save an edited row.

use rekall_kernel::{Id, Instant};
use rekall_store::company;
use rekall_store::repository as repo;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

async fn a_company(db: &impl sea_orm::ConnectionTrait, created: Instant) -> company::Model {
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: created, updated_at: created };
    company.clone().into_active_model().insert(db).await.unwrap();
    company
}

#[tokio::test]
async fn an_unchanged_row_is_not_written_and_keeps_its_stamp() {
    let database = rekall_store::open_temporary().await.unwrap();
    let created = Instant::parse("2026-01-01T00:00:00Z").unwrap();
    let before = a_company(&database.conn, created).await;

    let saved = repo::update_if_changed(&database.conn, &before, before.clone(), Instant::now()).await.unwrap();

    assert_eq!(saved.updated_at, created);
    assert_eq!(repo::company::require(&database.conn, before.id).await.unwrap().updated_at, created);
}

#[tokio::test]
async fn a_changed_row_is_stamped_and_written() {
    let database = rekall_store::open_temporary().await.unwrap();
    let before = a_company(&database.conn, Instant::parse("2026-01-01T00:00:00Z").unwrap()).await;
    let now = Instant::parse("2026-02-01T00:00:00Z").unwrap();
    let mut after = before.clone();
    after.name = "Globex".into();

    let saved = repo::update_if_changed(&database.conn, &before, after, now).await.unwrap();

    let stored = repo::company::require(&database.conn, before.id).await.unwrap();
    assert_eq!(stored, saved);
    assert_eq!((stored.name.as_str(), stored.updated_at), ("Globex", now));
}

#[tokio::test]
async fn a_change_that_breaks_a_rule_is_refused_and_nothing_is_written() {
    let database = rekall_store::open_temporary().await.unwrap();
    let before = a_company(&database.conn, Instant::now()).await;
    let mut after = before.clone();
    after.name = " ".into();

    let refused = repo::update_if_changed(&database.conn, &before, after, Instant::now()).await.unwrap_err();

    assert!(refused.is_illegal_argument());
    assert_eq!(repo::company::require(&database.conn, before.id).await.unwrap().name, "Acme");
}

#[tokio::test]
async fn require_reports_a_missing_row_as_not_found() {
    let database = rekall_store::open_temporary().await.unwrap();
    let missing = Id::random();

    let refused = repo::company::require(&database.conn, missing).await.unwrap_err();

    assert_eq!(refused, rekall_kernel::RekallError::not_found("Company", missing));
}
