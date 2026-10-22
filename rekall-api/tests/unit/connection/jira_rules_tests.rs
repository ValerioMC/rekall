use super::*;

fn issue(key: &str, summary: &str, description: Option<&str>) -> JiraIssue {
    JiraIssue { key: key.into(), summary: summary.into(), status: None, issue_type: None, description: description.map(str::to_string) }
}

#[test]
fn a_project_key_is_upper_cased_and_anything_else_is_refused() {
    assert_eq!(project_key_of(Some(" eng ")).unwrap(), "ENG");
    assert_eq!(project_key_of(Some("MY_APP2")).unwrap(), "MY_APP2");
    assert!(project_key_of(None).is_err());
    assert!(project_key_of(Some("group/app")).is_err());
    assert!(project_key_of(Some("2ENG")).is_err());
}

#[test]
fn the_open_issues_are_the_ones_not_done_most_recently_updated_first() {
    assert_eq!(open_issues_query("ENG"), "project = \"ENG\" AND statusCategory != Done ORDER BY updated DESC");
}

#[test]
fn chosen_keys_are_upper_cased_deduplicated_and_kept_to_the_bound_project() {
    let keys = vec!["eng-2".to_string(), "ENG-10".to_string(), "ENG-2".to_string()];
    assert_eq!(chosen_issues_query("ENG", &keys).unwrap(), "key in (ENG-10, ENG-2) ORDER BY key ASC");
    assert!(chosen_issues_query("ENG", &["OPS-1".to_string()]).is_err());
    assert!(chosen_issues_query("ENG", &["ENG-1) OR project = OPS".to_string()]).is_err());
    assert!(chosen_issues_query("ENG", &["ENGX-1".to_string()]).is_err());
}

#[test]
fn an_import_takes_at_least_one_key_and_at_most_the_limit() {
    assert!(chosen_issues_query("ENG", &[]).is_err());
    let too_many: Vec<String> = (1..=u32::from(ISSUE_LIMIT) + 1).map(|number| format!("ENG-{number}")).collect();
    assert!(chosen_issues_query("ENG", &too_many).is_err());
}

#[test]
fn an_issue_becomes_a_task_labelled_by_its_key_that_links_back_to_it() {
    let project_id = Id::random();
    let request = task_request(project_id, "https://example.atlassian.net/", &issue("ENG-7", " Fix login ", Some("It fails.")));

    assert_eq!(request.label.as_deref(), Some("eng-7"));
    assert_eq!(request.title.as_deref(), Some("Fix login"));
    assert_eq!(request.description.as_deref(), Some("Imported from Jira: [ENG-7](https://example.atlassian.net/browse/ENG-7).\n\nIt fails."));
    assert_eq!(request.project_id, Some(project_id));
}

#[test]
fn an_issue_without_a_summary_is_titled_by_its_key_and_a_long_one_is_cut() {
    assert_eq!(task_request(Id::random(), "https://x", &issue("ENG-1", " ", None)).title.as_deref(), Some("ENG-1"));
    let long = "a".repeat(300);
    assert_eq!(task_request(Id::random(), "https://x", &issue("ENG-1", &long, None)).title.unwrap().chars().count(), 200);
}
