use crate::commit::PendingChangeKind;
use super::*;

fn mixed() -> Vec<PendingChange> {
    vec![
        PendingChange::new(PendingChangeKind::Added, "src/export/ExportController.java"),
        PendingChange::new(PendingChangeKind::Modified, "README.md"),
    ]
}

#[test]
fn only_documentation_files_make_a_docs_commit_only_tests_a_test_commit() {
    let docs = [
        PendingChange::new(PendingChangeKind::Modified, "README.md"),
        PendingChange::new(PendingChangeKind::Added, "docs/export.txt"),
    ];
    assert_eq!(type_of("Update guide", &docs), "docs");
    let tests = [
        PendingChange::new(PendingChangeKind::Added, "src/test/java/dev/vega/ExportTest.java"),
        PendingChange::new(PendingChangeKind::Modified, "ui/tests/unit/Export.spec.ts"),
    ];
    assert_eq!(type_of("Cover export", &tests), "test");
    let mixed = [
        PendingChange::new(PendingChangeKind::Added, "src/test/java/dev/vega/ExportTest.java"),
        PendingChange::new(PendingChangeKind::Added, "src/main/java/dev/vega/Export.java"),
    ];
    assert_eq!(type_of("Cover export", &mixed), "feat");
}

#[test]
fn the_title_decides_fix_and_refactor_in_english_or_italian() {
    assert_eq!(type_of("Fix the await review", &mixed()), "fix");
    assert_eq!(type_of("Errore quando faccio accetto", &mixed()), "fix");
    assert_eq!(type_of("Refactor the store", &mixed()), "refactor");
    assert_eq!(type_of("Fixture loader", &mixed()), "feat");
}
