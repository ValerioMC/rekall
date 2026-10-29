use std::collections::BTreeMap;

use rekall_kernel::Id;
use rekall_core::inventory::InventoryWritten;
use serde_json::json;

use super::*;

fn recorded(stored: usize, repeats: usize, settled_exposures: usize) -> InventoryRecorded {
    let by_ecosystem = BTreeMap::from([("npm", stored - 1), ("pip", 1)]);
    InventoryRecorded { written: InventoryWritten { project_id: Id::random(), project_anchor: "project:vega".into(), stored, repeats, by_ecosystem }, settled_exposures }
}

#[test]
fn the_list_arrives_as_an_array_or_as_a_string_holding_one() {
    let entry = json!({"ecosystem": "npm", "name": "lodash", "version": "4.17.21", "manifest": "package-lock.json"});
    let from_array = declared_of(Some(&json!([entry.clone()]))).unwrap();
    let from_text = declared_of(Some(&Value::String(json!([entry]).to_string()))).unwrap();
    assert_eq!(from_array, from_text);
    assert_eq!(from_array[0].name.as_deref(), Some("lodash"));
}

#[test]
fn a_list_that_is_missing_or_not_objects_is_refused() {
    assert!(matches!(declared_of(None), Err(ToolError::Illegal(message)) if message.contains("needs its `dependencies`")));
    assert!(matches!(declared_of(Some(&json!("lodash"))), Err(ToolError::Illegal(message)) if message.contains("{ecosystem, name, version, manifest}")));
}

#[test]
fn the_report_counts_per_ecosystem_and_says_what_was_folded_and_settled() {
    let quiet = report(&recorded(3, 0, 0));
    assert!(quiet.starts_with("Inventory of `project:vega` recorded: 3 dependencies (2 npm, 1 pip)."), "{quiet}");
    assert!(!quiet.contains("repeated") && !quiet.contains("warning"));
    let busy = report(&recorded(3, 2, 1));
    assert!(busy.contains("2 entries repeated others") && busy.contains("One exposure whose package is gone"), "{busy}");
}
