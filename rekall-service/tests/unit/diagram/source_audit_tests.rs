use std::fs;

use rekall_diagram::GraphCodec;
use serde_json::json;

use super::*;

fn project() -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    fs::create_dir_all(folder.path().join("src")).unwrap();
    let body: String = (1..=30).map(|n| format!("line {n}\n")).collect();
    fs::write(folder.path().join("src/order.rs"), body).unwrap();
    folder
}

fn graph(sources: serde_json::Value) -> SemanticGraph {
    GraphCodec::from_value(json!({
        "format": "rekall.semantic-graph",
        "version": 1,
        "nodes": [{"id": "validate", "kind": "action", "title": "Validate order", "sources": sources}]
    }))
    .unwrap()
}

fn audit(folder: &tempfile::TempDir, sources: serde_json::Value) -> std::result::Result<(), GraphViolations> {
    SourceAudit::check(&ProjectFolder::open(folder.path()).unwrap(), &graph(sources))
}

#[test]
fn spans_inside_files_of_the_folder_pass() {
    let folder = project();
    let sources = json!([{"file": "./src/order.rs", "startLine": 1, "endLine": 30}, {"file": "src/order.rs"}]);
    assert_eq!(audit(&folder, sources), Ok(()));
}

#[test]
fn a_file_that_is_not_there_is_refused_at_its_path() {
    let folder = project();
    let violations = audit(&folder, json!([{"file": "src/ghost.rs", "startLine": 3}])).unwrap_err();
    assert_eq!(violations.0.len(), 1);
    assert_eq!(violations.0[0].path, "nodes[0].sources[0].file");
    assert!(violations.0[0].message.contains("src/ghost.rs"), "{}", violations.0[0].message);
}

#[test]
fn lines_past_the_end_of_the_file_are_refused_each() {
    let folder = project();
    let violations = audit(&folder, json!([{"file": "src/order.rs", "startLine": 31, "endLine": 40}])).unwrap_err();
    let paths: Vec<&str> = violations.0.iter().map(|violation| violation.path.as_str()).collect();
    assert_eq!(paths, ["nodes[0].sources[0].startLine", "nodes[0].sources[0].endLine"]);
    assert!(violations.0[1].message.contains("(30 lines)"), "{}", violations.0[1].message);
}

#[test]
fn a_file_outside_the_folder_is_refused() {
    let folder = project();
    let outside = tempfile::NamedTempFile::new().unwrap();
    let violations = audit(&folder, json!([{"file": outside.path().to_string_lossy()}])).unwrap_err();
    assert!(violations.0[0].message.contains("outside the project folder"), "{}", violations.0[0].message);
}
