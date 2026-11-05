use super::*;

fn entry(seq: i32, kind: FeedbackKind, state: FeedbackState, quoted: Option<&str>, body: &str, reply: Option<&str>) -> FeedbackView {
    FeedbackView {
        seq,
        kind,
        state,
        quoted_markdown: quoted.map(str::to_string),
        body_markdown: body.into(),
        reply_markdown: reply.map(str::to_string),
        pass: None,
        taken_at: None,
        follows_seq: None,
    }
}

#[test]
fn a_pending_entry_is_rendered_in_full_with_its_seq_and_kind() {
    let rendered = ContextRenderer.render_thread(&[entry(3, FeedbackKind::Plan, FeedbackState::Pending, None, "Split step 2.\n\nIt does two jobs.", None)]);

    assert!(rendered.contains("<thread pending=\"1\" running=\"0\" applied=\"0\">"), "{rendered}");
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

    assert!(rendered.contains("<thread pending=\"0\" running=\"0\" applied=\"1\">"), "{rendered}");
    assert!(rendered.contains("<entry seq=\"1\" kind=\"IMPLEMENTATION\" state=\"APPLIED\">Fixed the totals in the report view.</entry>"), "{rendered}");
    assert!(!rendered.contains("long body"), "{rendered}");
}

#[test]
fn an_empty_thread_renders_nothing() {
    assert_eq!(ContextRenderer.render_thread(&[]), "");
}

#[test]
fn a_running_entry_is_rendered_in_full_with_its_pass_and_counted_apart() {
    let running = FeedbackView { pass: Some(2), ..entry(4, FeedbackKind::Implementation, FeedbackState::Running, None, "The button label is wrong.", None) };
    let waiting = entry(5, FeedbackKind::Plan, FeedbackState::Pending, None, "Split step 4.", None);

    let rendered = ContextRenderer.render_thread(&[running, waiting]);

    assert!(rendered.contains("<thread pending=\"1\" running=\"1\" applied=\"0\">"), "{rendered}");
    assert!(rendered.contains("<entry seq=\"4\" kind=\"IMPLEMENTATION\" state=\"RUNNING\" pass=\"2\">\nThe button label is wrong.\n</entry>"), "{rendered}");
}

#[test]
fn a_follow_up_names_the_entry_it_continues() {
    let applied = entry(1, FeedbackKind::Implementation, FeedbackState::Applied, None, "The totals are wrong.", Some("Fixed the totals."));
    let follow_up = FeedbackView { follows_seq: Some(1), ..entry(2, FeedbackKind::Implementation, FeedbackState::Pending, None, "Still wrong for refunds.", None) };

    let rendered = ContextRenderer.render_thread(&[applied, follow_up]);

    assert!(rendered.contains("<entry seq=\"2\" kind=\"IMPLEMENTATION\" state=\"PENDING\" follows=\"1\">\nStill wrong for refunds.\n</entry>"), "{rendered}");
    assert!(rendered.contains("<entry seq=\"1\" kind=\"IMPLEMENTATION\" state=\"APPLIED\">Fixed the totals.</entry>"), "{rendered}");
}
