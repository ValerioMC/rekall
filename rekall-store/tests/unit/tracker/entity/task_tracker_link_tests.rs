use rekall_kernel::{Id, Instant};

use super::*;

fn link_holding(fingerprint: Option<String>) -> Model {
    let now = Instant::now();
    Model { task_id: Id::random(), issue_key: Some("12".into()), issue_url: None, sync_error: None, synced_fingerprint: fingerprint, created_at: now, updated_at: now }
}

fn task_titled(title: &str, description: Option<&str>) -> crate::task::Model {
    let mut task = crate::task::Model::new("pay".into(), title.into(), Id::random(), Instant::now());
    task.description = description.map(str::to_string);
    task
}

#[test]
fn the_fingerprint_of_a_content_never_changes_between_releases() {
    assert_eq!(content_fingerprint("Pay", "Before Friday."), "4f510d3eeeb75b77");
}

#[test]
fn a_title_and_a_description_do_not_trade_characters() {
    assert_ne!(content_fingerprint("ab", "c"), content_fingerprint("a", "bc"));
}

#[test]
fn a_link_holds_the_content_it_was_written_with_until_the_task_changes() {
    let task = task_titled("Pay", Some("Before Friday."));
    let link = link_holding(Some(content_fingerprint("Pay", "Before Friday.")));

    assert!(link.holds_content_of(&task));
    assert!(!link.holds_content_of(&task_titled("Pay", Some("Before Monday."))));
    assert!(!link.holds_content_of(&task_titled("Pay the invoice", Some("Before Friday."))));
}

#[test]
fn a_link_that_was_never_written_holds_nothing() {
    assert!(!link_holding(None).holds_content_of(&task_titled("Pay", None)));
}

#[test]
fn a_task_without_a_description_matches_a_link_written_with_an_empty_one() {
    assert!(link_holding(Some(content_fingerprint("Pay", ""))).holds_content_of(&task_titled("Pay", None)));
}
