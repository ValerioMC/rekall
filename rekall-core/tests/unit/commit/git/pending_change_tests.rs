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
fn a_rename_is_the_new_path_renamed_and_the_old_path_deleted() {
    let changes = PendingChange::parse_all("R  new.ts\0old.ts\0 M kept.ts\0").unwrap();
    assert_eq!(
        changes,
        vec![
            PendingChange::new(PendingChangeKind::Renamed, "new.ts"),
            PendingChange::new(PendingChangeKind::Deleted, "old.ts"),
            PendingChange::new(PendingChangeKind::Modified, "kept.ts"),
        ]
    );
}

#[test]
fn paths_with_spaces_and_accents_arrive_unquoted() {
    let changes = PendingChange::parse_all("?? a b.txt\0?? é.txt\0").unwrap();
    assert_eq!(changes.iter().map(|change| change.path.as_str()).collect::<Vec<_>>(), vec!["a b.txt", "é.txt"]);
}

#[test]
fn nothing_pending_is_no_change() {
    assert!(PendingChange::parse_all("").unwrap().is_empty());
}

#[test]
fn a_line_too_short_to_be_a_status_line_is_refused() {
    let error = PendingChange::parse("M ").unwrap_err();
    assert!(error.message().contains("Unexpected git status line"));
}
