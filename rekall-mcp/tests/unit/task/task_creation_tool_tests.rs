use rekall_service::task::StepDraft;
use serde_json::json;

use super::{drafts_of, project_label};
use crate::tool::Anchor;

#[test]
fn no_steps_argument_reads_as_no_drafts() {
    assert_eq!(drafts_of(None).unwrap(), Vec::new());
}

#[test]
fn steps_arrive_as_a_list_or_as_a_string_holding_one() {
    let expected = vec![
        StepDraft { title: "One".into(), detail: Some("Do it".into()) },
        StepDraft { title: "Two".into(), detail: None },
    ];
    let list = json!([{ "title": "One", "detail": "Do it" }, { "title": "Two" }]);

    assert_eq!(drafts_of(Some(&list)).unwrap(), expected);
    assert_eq!(drafts_of(Some(&json!(list.to_string()))).unwrap(), expected);
}

#[test]
fn a_step_without_a_title_is_refused_and_says_what_was_expected() {
    let refused = drafts_of(Some(&json!([{ "detail": "no title" }]))).unwrap_err();

    assert!(format!("{refused:?}").contains("{title, detail}"), "{refused:?}");
}

#[test]
fn only_a_lone_project_anchor_names_where_the_task_goes() {
    assert_eq!(project_label(&Anchor::parse_all(Some("project:vega")).unwrap()).unwrap(), "vega");
    assert!(project_label(&Anchor::parse_all(Some("project:vega task:report")).unwrap()).is_err());
    assert!(project_label(&Anchor::parse_all(Some("vega")).unwrap()).is_err());
}

#[test]
fn the_kind_is_a_task_unless_an_epic_is_asked_for_in_any_case() {
    use rekall_service::task::TaskKind;

    assert_eq!(super::kind_of(None).unwrap(), TaskKind::Task);
    assert_eq!(super::kind_of(Some("Epic")).unwrap(), TaskKind::Epic);
    assert!(super::kind_of(Some("story")).is_err());
}

#[test]
fn an_epic_created_is_answered_with_how_to_file_tasks_under_it() {
    let created = rekall_service::task::Created {
        title: "Checkout".into(),
        kind: rekall_service::task::TaskKind::Epic,
        task_anchor: "project:vega task:checkout".into(),
        drafts: 0,
    };

    let answer = super::report(&created);

    assert!(answer.starts_with("Epic \"Checkout\" created as `project:vega task:checkout`."), "{answer}");
    assert!(answer.contains("`rekall_join_epic` and the anchor `project:vega task:checkout`"), "{answer}");
}
