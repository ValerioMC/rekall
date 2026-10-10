use serde_json::json;

use super::*;

#[test]
fn reading_trims_titles_keys_and_the_blank_lines_around_the_mermaid() {
    let document = DocumentCodec::from_value(json!({
        "sheets": [{
            "title": "  ",
            "mermaid": "\n\nsequenceDiagram\n  A->>B: hi\n\n",
            "links": { " A ": [{ "file": "src/a.rs" }] }
        }]
    }))
    .unwrap();
    let sheet = &document.sheets[0];
    assert_eq!(sheet.title, None);
    assert_eq!(sheet.mermaid, "sequenceDiagram\n  A->>B: hi");
    assert!(sheet.links.contains_key("A"));
}

#[test]
fn a_field_the_format_does_not_define_is_refused() {
    let refused = DocumentCodec::from_value(json!({ "sheets": [{ "mermaid": "pie", "kind": "pie" }] }));
    assert!(refused.unwrap_err().to_string().contains("unknown field `kind`"));
}

#[test]
fn what_is_written_reads_back_the_same() {
    let document = DocumentCodec::from_value(json!({
        "sheets": [{ "mermaid": "flowchart LR\n A-->B", "links": { "A": [{ "file": "a.rs", "startLine": 2, "endLine": 9, "symbol": "a" }] } }]
    }))
    .unwrap();
    assert_eq!(DocumentCodec::parse(&DocumentCodec::write(&document)).unwrap(), document);
}
