use super::*;

#[test]
fn headings_paragraphs_and_inline_marks_become_markdown() {
    let markdown = to_markdown("<h2>Overview</h2><p>The <strong>core</strong> is <em>small</em> and uses <code>axum</code> &amp; tokio.</p>");
    assert_eq!(markdown, "## Overview\n\nThe **core** is *small* and uses `axum` & tokio.");
}

#[test]
fn nested_lists_keep_their_markers_and_indentation() {
    let markdown = to_markdown("<ul><li>one<ol><li>first</li><li>second</li></ol></li><li>two</li></ul>");
    assert_eq!(markdown, "- one\n  1. first\n  2. second\n- two");
}

#[test]
fn a_link_keeps_its_text_and_address() {
    assert_eq!(to_markdown(r#"<p>See <a href="https://example.com/a?x=1&amp;y=2">the guide</a>.</p>"#), "See [the guide](https://example.com/a?x=1&y=2).");
}

#[test]
fn a_code_macro_becomes_a_fenced_block_and_its_parameters_are_dropped() {
    let storage = r#"<ac:structured-macro ac:name="code"><ac:parameter ac:name="language">bash</ac:parameter><ac:plain-text-body><![CDATA[make deploy
make verify]]></ac:plain-text-body></ac:structured-macro>"#;
    assert_eq!(to_markdown(storage), "```\nmake deploy\nmake verify\n```");
}

#[test]
fn a_table_gets_a_header_separator_after_its_first_row() {
    let markdown = to_markdown("<table><tbody><tr><th>Env</th><th>URL</th></tr><tr><td>prod</td><td>example.com</td></tr></tbody></table>");
    assert_eq!(markdown, "| Env | URL |\n|---|---|\n| prod | example.com |");
}

#[test]
fn an_unknown_tag_is_dropped_and_its_text_kept() {
    assert_eq!(to_markdown("<p><ac:inline-comment-marker>marked</ac:inline-comment-marker> text<br />next</p>"), "marked text\nnext");
}
