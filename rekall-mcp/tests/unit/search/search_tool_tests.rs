use rekall_core::search::{SearchHit, SearchHitKind};

use super::{is_too_short, project_of, report, too_short_message};

fn hit(kind: SearchHitKind, title: &str, anchor: &str, excerpt: &str) -> SearchHit {
    SearchHit { kind, task_id: None, step_id: None, document_id: None, title: title.into(), r#where: anchor.into(), context_anchor: anchor.into(), excerpt: excerpt.into() }
}

#[test]
fn a_result_shows_its_kind_title_anchor_and_excerpt_and_nothing_more() {
    let found = report("pdf", Some("vega"), &[hit(SearchHitKind::Wrapup, "Export", "project:vega task:export", "…renders the PDF…")]);

    assert!(found.starts_with("1 result for \"pdf\" in `project:vega`."), "{found}");
    assert!(found.contains("- wrapup \"Export\": `project:vega task:export`\n  …renders the PDF…"), "{found}");
}

#[test]
fn a_note_is_loaded_by_its_own_anchor_not_by_the_task_it_opens_on() {
    let mut note = hit(SearchHitKind::Note, "ledger.md", "project:vega task:checkout", "the ledger rules");
    note.context_anchor = "note:3f2a9c1e".into();

    let found = report("ledger", None, &[note]);

    assert!(found.contains("`note:3f2a9c1e`"), "{found}");
    assert!(!found.contains("task:checkout"), "{found}");
}

#[test]
fn no_result_says_where_it_looked() {
    assert_eq!(report("pdf", None, &[]), "Nothing in every project matches \"pdf\".");
    assert_eq!(report("pdf", Some("vega"), &[]), "Nothing in `project:vega` matches \"pdf\".");
}

#[test]
fn a_term_under_three_characters_is_answered_with_a_message_not_an_error() {
    assert!(is_too_short("ab"));
    assert!(is_too_short("  ab  "));
    assert!(!is_too_short("abc"));
    assert!(too_short_message().contains("at least 3 characters"));
}

#[test]
fn only_a_project_anchor_narrows_the_search() {
    assert_eq!(project_of(None).unwrap(), None);
    assert_eq!(project_of(Some("project:vega")).unwrap().as_deref(), Some("vega"));
    assert_eq!(project_of(Some("project:vega task:report")).unwrap().as_deref(), Some("vega"));
    assert!(project_of(Some("company:acme")).is_err());
}
