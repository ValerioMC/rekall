use rekall_core::component::ComponentWritten;

use super::report;

fn written(replaced: bool, components_on_project: usize) -> ComponentWritten {
    ComponentWritten { name: "Billing".into(), project_anchor: "project:vega".into(), components_on_project, replaced }
}

#[test]
fn a_new_component_is_reported_with_the_count_the_project_now_has() {
    let said = report(&written(false, 3));

    assert!(said.contains("Component \"Billing\" written on `project:vega`"), "{said}");
    assert!(said.contains("3 components"), "{said}");
}

#[test]
fn a_rewritten_component_warns_that_its_earlier_text_is_gone() {
    let said = report(&written(true, 1));

    assert!(said.contains("rewritten"), "{said}");
    assert!(said.contains("earlier text is gone"), "{said}");
    assert!(said.contains("1 component."), "{said}");
}
