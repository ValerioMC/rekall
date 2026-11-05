use super::*;

fn a_revision() -> Model {
    Model::new(Id::random(), 1, FeedbackKind::Revision, Some("before".into()), "after".into(), Instant::now())
}

fn a_correction() -> Model {
    Model::new(Id::random(), 1, FeedbackKind::Implementation, None, "the totals are wrong".into(), Instant::now())
}

#[test]
fn a_new_entry_waits_for_a_session() {
    let entry = a_correction();
    assert!(entry.pending());
    assert_eq!((entry.reply_markdown, entry.applied_at), (None, None));
}

#[test]
fn a_revision_and_a_correction_that_follow_the_rules_pass() {
    assert!(a_revision().validate().is_ok());
    assert!(a_correction().validate().is_ok());
}

#[test]
fn a_revision_without_a_quoted_text_is_refused() {
    let mut entry = a_revision();
    entry.quoted_markdown = None;
    let refused = entry.validate().unwrap_err();
    assert!(refused.is_illegal_argument());
    assert!(refused.message().contains("a revision must"), "{}", refused.message());
}

#[test]
fn a_correction_that_quotes_a_text_is_refused() {
    let mut entry = a_correction();
    entry.quoted_markdown = Some("before".into());
    assert!(entry.validate().unwrap_err().is_illegal_argument());
}

#[test]
fn a_blank_body_is_refused() {
    let mut entry = a_correction();
    entry.body_markdown = "  ".into();
    assert!(entry.validate().unwrap_err().message().contains("bodyMarkdown must not be blank"));
}

#[test]
fn a_reply_over_the_cap_is_refused_and_one_at_the_cap_passes() {
    let mut entry = a_correction();
    entry.reply_markdown = Some("x".repeat(REPLY_MAX_CHARACTERS));
    assert!(entry.validate().is_ok());
    entry.reply_markdown = Some("x".repeat(REPLY_MAX_CHARACTERS + 1));
    assert!(entry.validate().unwrap_err().message().contains("replyMarkdown is capped at 2000"));
}

#[test]
fn applying_an_entry_keeps_the_reply_and_the_moment() {
    let mut entry = a_correction();
    let now = Instant::now();
    entry.mark_applied("Fixed the totals in the report view.".into(), now);
    assert!(!entry.pending());
    assert_eq!(entry.reply_markdown.as_deref(), Some("Fixed the totals in the report view."));
    assert_eq!(entry.applied_at, Some(now));
}
