use super::super::PendingChangeKind;
use super::*;

const ANCHOR: &str = "project:vega task:report-builder";
const DETAIL: &str = "## What this step does\n\nExpose **the weekly report** as a download, so the finance team stops asking for it by mail.\nThe endpoint streams the file instead of building it in memory.\n\n```java\nignored();\n```\n\nIt is read-only and needs no migration.\n";

fn step() -> Subject {
    Subject { title: "Wire the export endpoint".into(), anchor: ANCHOR.into(), step_number: Some(3), description: Some(DETAIL.into()) }
}

fn mixed() -> Vec<PendingChange> {
    vec![
        PendingChange::new(PendingChangeKind::Added, "src/export/ExportController.java"),
        PendingChange::new(PendingChangeKind::Modified, "README.md"),
    ]
}

#[test]
fn without_a_session_message_the_body_is_the_steps_detail_as_prose_and_no_file_is_listed() {
    let message = CommitMessageGenerator::generate(&step(), &mixed(), None);
    assert!(message.starts_with("feat: Wire the export endpoint\n\n"), "{message}");
    assert!(message.contains("Expose the weekly report as a download"));
    assert!(message.contains("It is read-only and needs no migration."));
    assert!(!message.contains("##") && !message.contains("**") && !message.contains("ignored();"));
    assert!(!message.contains("ExportController.java") && !message.contains("README.md") && !message.contains("files"));
    assert!(message.ends_with("\n\nRefs: project:vega task:report-builder, step 3\n"));
}

#[test]
fn a_session_message_wins() {
    let written = "fix: stream the weekly report instead of buffering it\n\nThe export built the whole file in memory and ran out of heap on large months.\nIt now writes rows as they are read.\n";
    let message = CommitMessageGenerator::generate(&step(), &mixed(), Some(written));
    assert!(message.starts_with("fix: stream the weekly report instead of buffering it\n\n"));
    assert!(
        message.contains("ran out of heap on large\nmonths. It now writes rows as they are read."),
        "the session's lines are reflowed at 72 columns: {message}"
    );
    assert!(!message.contains("Expose the weekly"));
    assert!(message.ends_with("Refs: project:vega task:report-builder, step 3\n"));
}

#[test]
fn a_bare_session_subject_gets_a_type_and_keeps_the_derived_body() {
    let message = CommitMessageGenerator::generate(&step(), &mixed(), Some("Stream the weekly report"));
    assert!(message.starts_with("feat: Stream the weekly report\n\n"));
    assert!(message.contains("Expose the weekly report"));
}

#[test]
fn body_lines_are_wrapped_at_72_columns_and_list_items_keep_a_hanging_indent() {
    let written = format!("feat: x\n\n{}\n\n- {}", "word ".repeat(40), "item ".repeat(30));
    let message = CommitMessageGenerator::generate(&step(), &mixed(), Some(&written));
    assert!(message.split('\n').all(|line| jstr::len(line) <= 72));
    assert!(message.contains("\n- item") && message.contains("\n  item"));
}

#[test]
fn a_long_detail_is_cut_at_a_sentence() {
    let detail = "This sentence is long enough to matter. ".repeat(40);
    let body = CommitMessageGenerator::derived_body(Some(&detail));
    assert!(jstr::len(&body) <= DERIVED_BODY_MAX);
    assert!(body.ends_with("matter."));
}

#[test]
fn a_task_claim_with_no_wrapup_has_a_subject_and_the_refs_line() {
    let task = Subject { title: "Report builder".into(), anchor: ANCHOR.into(), step_number: None, description: None };
    let message = CommitMessageGenerator::generate(&task, &[PendingChange::new(PendingChangeKind::Deleted, "old.js")], None);
    assert_eq!(message, "feat: Report builder\n\nRefs: project:vega task:report-builder\n");
}

#[test]
fn only_documentation_files_make_a_docs_commit_only_tests_a_test_commit() {
    let docs = [
        PendingChange::new(PendingChangeKind::Modified, "README.md"),
        PendingChange::new(PendingChangeKind::Added, "docs/export.txt"),
    ];
    assert_eq!(CommitMessageGenerator::type_of("Update guide", &docs), "docs");
    let tests = [
        PendingChange::new(PendingChangeKind::Added, "src/test/java/dev/vega/ExportTest.java"),
        PendingChange::new(PendingChangeKind::Modified, "ui/tests/unit/Export.spec.ts"),
    ];
    assert_eq!(CommitMessageGenerator::type_of("Cover export", &tests), "test");
    let mixed = [
        PendingChange::new(PendingChangeKind::Added, "src/test/java/dev/vega/ExportTest.java"),
        PendingChange::new(PendingChangeKind::Added, "src/main/java/dev/vega/Export.java"),
    ];
    assert_eq!(CommitMessageGenerator::type_of("Cover export", &mixed), "feat");
}

#[test]
fn the_title_decides_fix_and_refactor_in_english_or_italian() {
    assert_eq!(CommitMessageGenerator::type_of("Fix the await review", &mixed()), "fix");
    assert_eq!(CommitMessageGenerator::type_of("Errore quando faccio accetto", &mixed()), "fix");
    assert_eq!(CommitMessageGenerator::type_of("Refactor the store", &mixed()), "refactor");
    assert_eq!(CommitMessageGenerator::type_of("Fixture loader", &mixed()), "feat");
}

#[test]
fn the_subject_line_is_cut_at_72_characters_with_the_title_ellipsised() {
    let long = Subject { title: "A".repeat(100), anchor: ANCHOR.into(), step_number: Some(1), description: None };
    let subject = CommitMessageGenerator::subject_line(&long, &[]);
    assert_eq!(jstr::len(&subject), SUBJECT_MAX);
    assert!(subject.starts_with("feat: AAAA") && subject.ends_with('…'));
}

#[test]
fn a_whitespace_heavy_title_is_collapsed_to_single_spaces() {
    let messy = Subject { title: "  Wire   the\nexport  ".into(), anchor: ANCHOR.into(), step_number: Some(1), description: None };
    assert_eq!(CommitMessageGenerator::subject_line(&messy, &[]), "feat: Wire the export");
}
