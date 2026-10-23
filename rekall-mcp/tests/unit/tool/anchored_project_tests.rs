use super::*;

fn parsed(raw: &str) -> Result<AnchoredProject, ToolError> {
    AnchoredProject::from(&Anchor::parse_all(Some(raw)).unwrap())
}

#[test]
fn a_project_anchor_names_the_project_and_a_task_beside_it_is_allowed() {
    assert_eq!(parsed("project:vega").unwrap().project_label, "vega");
    assert_eq!(parsed("project:vega task:report").unwrap().project_label, "vega");
}

#[test]
fn anything_that_is_not_one_project_is_refused() {
    assert!(matches!(parsed("task:report"), Err(ToolError::Failure(message)) if message.contains("No project")));
    assert!(matches!(parsed("project:a project:b"), Err(ToolError::Failure(message)) if message.contains("Two project anchors")));
    assert!(matches!(parsed("company:acme project:vega"), Err(ToolError::Failure(message)) if message.contains("`company:acme` does not name a project")));
    assert!(matches!(parsed("vega"), Err(ToolError::Failure(message)) if message.contains("`vega` does not name a project")));
}
