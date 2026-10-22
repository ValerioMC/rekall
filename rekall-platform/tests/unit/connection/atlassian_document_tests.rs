use serde_json::json;

use super::*;

fn markdown(document: serde_json::Value) -> String {
    serde_json::from_value::<AdfNode>(document).unwrap().to_markdown()
}

fn text(value: &str) -> serde_json::Value {
    json!({ "type": "text", "text": value })
}

#[test]
fn paragraphs_and_headings_are_separated_by_a_blank_line() {
    let document = json!({ "type": "doc", "content": [
        { "type": "heading", "attrs": { "level": 2 }, "content": [text("Goal")] },
        { "type": "paragraph", "content": [text("First"), { "type": "hardBreak" }, text("second")] },
    ]});
    assert_eq!(markdown(document), "## Goal\n\nFirst\nsecond");
}

#[test]
fn lists_carry_their_markers_and_nest_by_indentation() {
    let item = |value: &str| json!({ "type": "listItem", "content": [{ "type": "paragraph", "content": [text(value)] }] });
    let nested = json!({ "type": "listItem", "content": [
        { "type": "paragraph", "content": [text("parent")] },
        { "type": "bulletList", "content": [item("child")] },
    ]});
    let document = json!({ "type": "doc", "content": [
        { "type": "bulletList", "content": [item("a"), nested] },
        { "type": "orderedList", "content": [item("one"), item("two")] },
    ]});
    assert_eq!(markdown(document), "- a\n- parent\n  - child\n\n1. one\n2. two");
}

#[test]
fn marks_become_markdown_and_code_keeps_its_language() {
    let document = json!({ "type": "doc", "content": [
        { "type": "paragraph", "content": [
            { "type": "text", "text": "run", "marks": [{ "type": "code" }] },
            text(" then "),
            { "type": "text", "text": "docs", "marks": [{ "type": "link", "attrs": { "href": "https://example.com" } }] },
        ]},
        { "type": "codeBlock", "attrs": { "language": "sh" }, "content": [text("make test")] },
    ]});
    assert_eq!(markdown(document), "`run` then [docs](https://example.com)\n\n```sh\nmake test\n```");
}

#[test]
fn an_unknown_node_is_written_as_its_content() {
    let document = json!({ "type": "doc", "content": [
        { "type": "panel", "attrs": { "panelType": "info" }, "content": [{ "type": "paragraph", "content": [text("Heads up")] }] },
    ]});
    assert_eq!(markdown(document), "Heads up");
}
