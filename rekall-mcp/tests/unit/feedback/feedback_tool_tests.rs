use serde_json::json;

use super::*;

#[test]
fn replies_name_an_entry_by_a_number_or_a_numeric_string() {
    let replies = json!([{ "seq": 2, "reply": "Split it." }, { "seq": "5", "reply": "Renamed." }]);

    assert_eq!(replies_of(Some(&replies)).unwrap(), vec![(2, "Split it.".to_string()), (5, "Renamed.".to_string())]);
}

#[test]
fn replies_arrive_as_a_list_or_as_a_string_holding_one() {
    let list = json!([{ "seq": 1, "reply": "Done." }]);

    assert_eq!(replies_of(Some(&list)).unwrap(), replies_of(Some(&json!(list.to_string()))).unwrap());
}

#[test]
fn missing_replies_and_a_seq_that_is_no_number_are_refused() {
    assert!(replies_of(None).is_err());
    let refused = replies_of(Some(&json!([{ "seq": "first", "reply": "Done." }]))).unwrap_err();
    assert!(format!("{refused:?}").contains("'seq' is the number of an entry"), "{refused:?}");
    assert!(replies_of(Some(&json!([{ "seq": 1 }]))).is_err());
}

#[test]
fn the_answer_lists_what_was_applied_and_what_is_still_pending() {
    let replied = Replied { task_anchor: "project:vega task:pay".into(), applied: vec![1, 3], still_pending: vec![2] };

    assert_eq!(report(&replied), "Replied to #1, #3 on `project:vega task:pay`. Still pending: #2.");
}

#[test]
fn the_answer_says_when_nothing_is_pending_any_more() {
    let replied = Replied { task_anchor: "project:vega task:pay".into(), applied: vec![1], still_pending: vec![] };

    assert_eq!(report(&replied), "Replied to #1 on `project:vega task:pay`. No entry is pending any more.");
}
