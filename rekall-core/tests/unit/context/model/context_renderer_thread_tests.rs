use super::*;

fn entry(seq: i32, kind: FeedbackKind, state: FeedbackState, quoted: Option<&str>, body: &str, reply: Option<&str>) -> FeedbackView {
    FeedbackView {
        seq,
        kind,
        state,
        quoted_markdown: quoted.map(str::to_string),
        body_markdown: body.into(),
        reply_markdown: reply.map(str::to_string),
    }
}

#[test]
fn a_pending_entry_is_rendered_in_full_with_its_seq_and_kind() {
    let rendered = ContextRenderer.render_thread(&[entry(3, FeedbackKind::Plan, FeedbackState::Pending, None, "Split step 2.\n\nIt does two jobs.", None)]);

    assert!(rendered.contains("<thread pending=\"1\" applied=\"0\">"), "{rendered}");
    assert!(rendered.contains("<entry seq=\"3\" kind=\"PLAN\" state=\"PENDING\">\nSplit step 2.\n\nIt does two jobs.\n</entry>"), "{rendered}");
}

#[test]
fn a_pending_revision_shows_the_text_before_and_the_text_now() {
    let rendered = ContextRenderer.render_thread(&[entry(1, FeedbackKind::Revision, FeedbackState::Pending, Some("Export to CSV."), "Export to CSV and XLSX.", None)]);

    assert!(rendered.contains("<quoted>\nExport to CSV.\n</quoted>\n<new>\nExport to CSV and XLSX.\n</new>"), "{rendered}");
}

#[test]
fn an_applied_entry_is_one_line_with_the_first_line_of_its_reply() {
    let rendered = ContextRenderer.render_thread(&[entry(
        1,
        FeedbackKind::Implementation,
        FeedbackState::Applied,
        None,
        "The totals are wrong and this long body must not travel.",
        Some("Fixed the totals in the report view.\nSecond line."),
    )]);

    assert!(rendered.contains("<thread pending=\"0\" applied=\"1\">"), "{rendered}");
    assert!(rendered.contains("<entry seq=\"1\" kind=\"IMPLEMENTATION\" state=\"APPLIED\">Fixed the totals in the report view.</entry>"), "{rendered}");
    assert!(!rendered.contains("long body"), "{rendered}");
}

#[test]
fn an_empty_thread_renders_nothing() {
    assert_eq!(ContextRenderer.render_thread(&[]), "");
}
