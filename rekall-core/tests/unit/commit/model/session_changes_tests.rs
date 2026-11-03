use super::*;
use crate::commit::PendingChangeKind;

fn modified(path: &str) -> PendingChange {
    PendingChange::new(PendingChangeKind::Modified, path)
}

fn kept(path: &str, hash: Option<&str>) -> (String, Option<String>) {
    (path.to_string(), hash.map(str::to_string))
}

#[test]
fn a_file_unchanged_since_the_start_is_left_out_and_one_edited_since_is_the_sessions() {
    let pending = vec![modified("notes.txt"), modified("src/lib.rs")];
    let hashes = vec![Some("aaa".to_string()), Some("new".to_string())];
    let snapshot = [kept("notes.txt", Some("aaa")), kept("src/lib.rs", Some("old"))];

    let split = SessionChanges::split(pending, hashes, &snapshot);

    assert_eq!(split.left_out, vec![modified("notes.txt")]);
    assert_eq!(split.own, vec![modified("src/lib.rs")]);
}

#[test]
fn a_file_the_snapshot_never_held_is_the_sessions() {
    let split = SessionChanges::split(vec![modified("fresh.txt")], vec![Some("aaa".to_string())], &[]);

    assert_eq!(split.own, vec![modified("fresh.txt")]);
    assert!(split.left_out.is_empty());
}

#[test]
fn a_file_deleted_before_and_still_deleted_is_left_out() {
    let gone = PendingChange::new(PendingChangeKind::Deleted, "gone.txt");

    let split = SessionChanges::split(vec![gone.clone()], vec![None], &[kept("gone.txt", None)]);

    assert_eq!(split.left_out, vec![gone]);
}

#[test]
fn a_file_deleted_at_the_start_and_written_back_since_is_the_sessions() {
    let split = SessionChanges::split(vec![modified("back.txt")], vec![Some("new".to_string())], &[kept("back.txt", None)]);

    assert_eq!(split.own, vec![modified("back.txt")]);
}

#[test]
fn without_a_snapshot_everything_is_the_sessions() {
    let split = SessionChanges::everything(vec![modified("a.txt"), modified("b.txt")]);

    assert_eq!(split.own.len(), 2);
    assert!(split.left_out.is_empty());
}
