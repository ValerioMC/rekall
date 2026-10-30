use rekall_core::flow::{PlannedRoute, PlannedStep};
use serde_json::json;

use super::{plan_or_empty, project_label};
use crate::tool::Anchor;

#[test]
fn no_steps_argument_reads_as_an_empty_plan() {
    assert!(plan_or_empty(None).unwrap().steps.is_empty());
}

#[test]
fn steps_arrive_with_their_sessions_and_routes_as_a_list_or_as_a_string_holding_one() {
    let expected = vec![
        PlannedStep { title: "One".into(), detail: Some("Do it".into()), routes: vec![], max_visits: None, starts_session: false },
        PlannedStep {
            title: "Two".into(),
            detail: None,
            routes: vec![PlannedRoute { outcome: "done".into(), to: None }],
            max_visits: None,
            starts_session: true,
        },
    ];
    let list = json!([{ "title": "One", "detail": "Do it" }, { "title": "Two", "new_session": true, "routes": [{ "outcome": "done", "to": "end" }] }]);

    assert_eq!(plan_or_empty(Some(&list)).unwrap().steps, expected);
    assert_eq!(plan_or_empty(Some(&json!(list.to_string()))).unwrap().steps, expected);
}

#[test]
fn a_step_without_a_title_is_refused_and_says_what_was_expected() {
    let refused = plan_or_empty(Some(&json!([{ "detail": "no title" }]))).unwrap_err();

    assert!(format!("{refused:?}").contains("{title, detail"), "{refused:?}");
}

#[test]
fn only_a_lone_project_anchor_names_where_the_task_goes() {
    assert_eq!(project_label(&Anchor::parse_all(Some("project:vega")).unwrap()).unwrap(), "vega");
    assert!(project_label(&Anchor::parse_all(Some("project:vega task:report")).unwrap()).is_err());
    assert!(project_label(&Anchor::parse_all(Some("vega")).unwrap()).is_err());
}

#[test]
fn the_kind_is_a_task_unless_an_epic_is_asked_for_in_any_case() {
    use rekall_core::task::TaskKind;

    assert_eq!(super::kind_of(None).unwrap(), TaskKind::Task);
    assert_eq!(super::kind_of(Some("Epic")).unwrap(), TaskKind::Epic);
    assert!(super::kind_of(Some("story")).is_err());
}

#[test]
fn an_epic_created_is_answered_with_how_to_file_tasks_under_it() {
    let created = rekall_core::task::Created {
        title: "Checkout".into(),
        kind: rekall_core::task::TaskKind::Epic,
        task_anchor: "project:vega task:checkout".into(),
        steps: 0,
        routes: 0,
    };

    let answer = super::report(&created);

    assert!(answer.starts_with("Epic \"Checkout\" created as `project:vega task:checkout`."), "{answer}");
    assert!(answer.contains("`rekall_join_epic` and the anchor `project:vega task:checkout`"), "{answer}");
}

#[test]
fn a_checklist_created_is_answered_as_ready_to_work_and_a_flow_with_its_routes() {
    let checklist = rekall_core::task::Created {
        title: "Report".into(),
        kind: rekall_core::task::TaskKind::Task,
        task_anchor: "project:vega task:report".into(),
        steps: 2,
        routes: 0,
    };
    let flow = rekall_core::task::Created { steps: 3, routes: 4, ..checklist.clone() };

    assert_eq!(
        super::report(&checklist),
        "Task \"Report\" created as `project:vega task:report` with 2 steps ready to work: none is a draft, so a session can start the first at once. `/rk project:vega task:report` loads it."
    );
    assert!(super::report(&flow).contains("as a flow of 3 steps joined by 4 routes, ready to work"), "{}", super::report(&flow));
}
