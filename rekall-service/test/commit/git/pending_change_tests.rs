use super::*;

#[test]
fn an_untracked_file_reads_as_added() {
    assert_eq!(PendingChange::parse("?? src/new.ts").unwrap(), PendingChange::new(PendingChangeKind::Added, "src/new.ts"));
}

#[test]
fn a_staged_addition_and_a_tracked_modification_keep_their_kind() {
    assert_eq!(PendingChange::parse("A  src/staged.ts").unwrap().kind, PendingChangeKind::Added);
    assert_eq!(PendingChange::parse(" M src/edited.ts").unwrap().kind, PendingChangeKind::Modified);
    assert_eq!(PendingChange::parse("MM src/both.ts").unwrap().kind, PendingChangeKind::Modified);
}

#[test]
fn a_deletion_staged_or_not_reads_as_deleted() {
    assert_eq!(PendingChange::parse(" D gone.ts").unwrap().kind, PendingChangeKind::Deleted);
    assert_eq!(PendingChange::parse("D  gone.ts").unwrap().kind, PendingChangeKind::Deleted);
}

#[test]
fn a_rename_keeps_the_new_path() {
    assert_eq!(PendingChange::parse("R  old.ts -> new.ts").unwrap(), PendingChange::new(PendingChangeKind::Renamed, "new.ts"));
}

#[test]
fn a_line_too_short_to_be_a_status_line_is_refused() {
    let error = PendingChange::parse("M ").unwrap_err();
    assert!(error.message().contains("Unexpected git status line"));
}
