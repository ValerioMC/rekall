use rekall_service::flow::{PlannedRoute, PlannedStep};
use serde_json::json;

use super::plan_of;

#[test]
fn routes_name_a_step_by_its_number_or_the_end() {
    let steps = json!([
        { "title": "Test", "detail": "Run them", "routes": [{ "outcome": "passed", "to": "end" }, { "outcome": "app-error", "to": 2 }] },
        { "title": "Fix", "routes": [{ "outcome": "fixed", "to": "1" }] }
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
            },
            PlannedStep { title: "Fix".into(), detail: None, routes: vec![PlannedRoute { outcome: "fixed".into(), to: Some(0) }] },
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
