use super::*;

fn joined(left: Option<&str>) -> EpicJoined {
    EpicJoined {
        task_anchor: "project:vega task:pay".into(),
        epic_anchor: "project:vega task:checkout".into(),
        epic_title: "Checkout".into(),
        left: left.map(str::to_string),
    }
}

#[test]
fn the_answer_names_the_task_and_the_epic() {
    assert_eq!(
        report(&joined(None)),
        "`project:vega task:pay` is filed under the epic \"Checkout\" (`project:vega task:checkout`). `/rk` on the task \
         now loads the epic with it, its description and notes as part of the brief."
    );
}

#[test]
fn the_answer_names_the_epic_a_moved_task_left() {
    assert!(report(&joined(Some("billing"))).contains("(`project:vega task:checkout`), out of `task:billing`."));
}
