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

#[test]
fn reading_trims_a_walkthrough_and_drops_its_blank_texts() {
    let document = DocumentCodec::from_value(json!({
        "sheets": [{
            "mermaid": "sequenceDiagram\n  A->>B: hi",
            "walkthrough": { "external": [" B "], "messages": { " #1 ": { "explain": "  ", "input": " id " } } }
        }]
    }))
    .unwrap();
    let walkthrough = document.sheets[0].walkthrough.as_ref().unwrap();
    assert_eq!(walkthrough.external, vec!["B"]);
    let message = &walkthrough.messages["#1"];
    assert_eq!((message.explain.as_deref(), message.input.as_deref()), (None, Some("id")));
    assert_eq!(DocumentCodec::parse(&DocumentCodec::write(&document)).unwrap(), document);
}

#[test]
fn a_scenario_is_trimmed_and_reads_back_the_same() {
    let document = DocumentCodec::from_value(json!({
        "scenario": { "situation": " You drag step 4 above step 2 ", "example": "RK-12, step 4", "outcome": "steps 1, 4, 2, 3" },
        "sheets": [{ "mermaid": "sequenceDiagram\n  A->>B: hi" }]
    }))
    .unwrap();
    assert_eq!(document.scenario.as_ref().unwrap().situation, "You drag step 4 above step 2");
    assert_eq!(DocumentCodec::parse(&DocumentCodec::write(&document)).unwrap(), document);
}

#[test]
fn a_document_stored_without_a_scenario_still_reads() {
    let document = DocumentCodec::parse(r#"{"sheets":[{"mermaid":"flowchart LR\n A-->B"}]}"#).unwrap();
    assert_eq!(document.scenario, None);
    assert!(!DocumentCodec::write(&document).contains("scenario"));
}
