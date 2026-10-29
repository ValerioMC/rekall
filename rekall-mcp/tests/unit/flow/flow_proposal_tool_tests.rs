use rekall_core::flow::{PlannedRoute, PlannedStep};
use rekall_core::step::VisitBudget;
use serde_json::json;

use super::plan_of;

#[test]
fn routes_name_a_step_by_its_number_or_the_end() {
    let steps = json!([
        { "title": "Test", "detail": "Run them", "routes": [{ "outcome": "passed", "to": "end" }, { "outcome": "app-error", "to": 2 }] },
        { "title": "Fix", "max_visits": 3, "routes": [{ "outcome": "fixed", "to": "1" }] }
    ]);

    let plan = plan_of(Some(&steps)).unwrap();

    assert_eq!(
        plan.steps,
        vec![
            PlannedStep {
                title: "Test".into(),
                detail: Some("Run them".into()),
                routes: vec![
                    PlannedRoute { outcome: "passed".into(), to: None },
                    PlannedRoute { outcome: "app-error".into(), to: Some(1) },
                ],
                max_visits: None,
                starts_session: false,
            },
            PlannedStep {
                title: "Fix".into(),
                detail: None,
                routes: vec![PlannedRoute { outcome: "fixed".into(), to: Some(0) }],
                max_visits: Some(VisitBudget::new(3).unwrap()),
                starts_session: false,
            },
        ]
    );
}

#[test]
fn steps_arrive_as_a_list_or_as_a_string_holding_one() {
    let list = json!([{ "title": "Ship", "routes": [{ "outcome": "shipped", "to": "End" }] }]);

    assert_eq!(plan_of(Some(&list)).unwrap(), plan_of(Some(&json!(list.to_string()))).unwrap());
}

#[test]
fn a_target_that_is_neither_a_number_nor_end_is_refused() {
    for to in [json!(0), json!("next")] {
        let steps = json!([{ "title": "Test", "routes": [{ "outcome": "passed", "to": to }] }]);
        let refused = plan_of(Some(&steps)).unwrap_err();
        assert!(format!("{refused:?}").contains("Test"), "{refused:?}");
    }
}

#[test]
fn missing_steps_are_refused() {
    assert!(plan_of(None).is_err());
}

#[test]
fn a_visit_budget_outside_its_range_is_refused() {
    let steps = json!([{ "title": "Fix", "max_visits": 0, "routes": [{ "outcome": "fixed", "to": "end" }] }]);

    assert!(format!("{:?}", plan_of(Some(&steps)).unwrap_err()).contains("runs from 1"));
}

#[test]
fn a_step_asks_for_a_fresh_session_and_otherwise_joins_the_running_one() {
    let steps = json!([
        { "title": "Move", "routes": [{ "outcome": "done", "to": 2 }] },
        { "title": "Docs", "new_session": true, "routes": [{ "outcome": "done", "to": "end" }] }
    ]);

    let plan = plan_of(Some(&steps)).unwrap();

    assert_eq!(plan.steps.iter().map(|step| step.starts_session).collect::<Vec<_>>(), [false, true]);
}
