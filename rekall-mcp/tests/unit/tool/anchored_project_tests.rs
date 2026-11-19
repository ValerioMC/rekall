use super::*;

fn parsed(raw: &str) -> Result<AnchoredProject, ToolError> {
    AnchoredProject::from(&Anchor::parse_all(Some(raw)).unwrap())
}

fn alone(raw: &str) -> Option<ProjectRef> {
    AnchoredProject::alone(&Anchor::parse_all(Some(raw)).unwrap())
}

#[test]
fn a_project_anchor_names_the_project_and_a_task_beside_it_is_allowed() {
    assert_eq!(parsed("project:vega").unwrap().project, ProjectRef::bare("vega"));
    assert_eq!(parsed("project:vega task:report").unwrap().project, ProjectRef::bare("vega"));
}

#[test]
fn a_company_beside_the_project_qualifies_it_in_either_order() {
    let qualified = ProjectRef::in_company("Acme", "vega");
    assert_eq!(parsed("company:Acme project:vega").unwrap().project, qualified);
    assert_eq!(parsed("project:vega company:Acme task:report").unwrap().project, qualified);
}

#[test]
fn anything_that_is_not_one_project_is_refused() {
    assert!(matches!(parsed("task:report"), Err(ToolError::Failure(message)) if message.contains("No project")));
    assert!(matches!(parsed("company:acme"), Err(ToolError::Failure(message)) if message.contains("No project")));
    assert!(matches!(parsed("project:a project:b"), Err(ToolError::Failure(message)) if message.contains("Two project anchors")));
    assert!(matches!(parsed("company:a company:b project:vega"), Err(ToolError::Failure(message)) if message.contains("Two company anchors")));
    assert!(matches!(parsed("note:3f2a9c1e project:vega"), Err(ToolError::Failure(message)) if message.contains("`note:3f2a9c1e` does not name a project")));
    assert!(matches!(parsed("vega"), Err(ToolError::Failure(message)) if message.contains("`vega` does not name a project")));
}

#[test]
fn alone_takes_the_project_with_or_without_its_company_and_nothing_else() {
    assert_eq!(alone("project:vega"), Some(ProjectRef::bare("vega")));
    assert_eq!(alone("company:Acme project:vega"), Some(ProjectRef::in_company("Acme", "vega")));
    assert_eq!(alone("project:vega task:report"), None);
    assert_eq!(alone("company:Acme"), None);
}
