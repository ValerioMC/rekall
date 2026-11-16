use rekall_core::component::{ComponentsRead, ProjectComponentView};
use rekall_kernel::{Id, Instant};

use super::report;

fn component(name: &str, folder: &str, summary: &str, description: &str) -> ProjectComponentView {
    let now = Instant::now();
    ProjectComponentView {
        id: Id::random(),
        project_id: Id::random(),
        name: name.into(),
        folder: folder.into(),
        summary: summary.into(),
        description: description.into(),
        position: 0,
        created_at: now,
        updated_at: now,
    }
}

fn read(components: Vec<ProjectComponentView>, asked_for_one: bool) -> ComponentsRead {
    ComponentsRead { project_anchor: "project:vega".into(), components, asked_for_one }
}

#[test]
fn without_a_name_the_components_are_listed_with_their_summary_and_size() {
    let listed = report(&read(vec![component("Billing", "/work/billing", "Charges customers.", "## Purpose"), component("Edge", "/work/edge", "Routes requests.", "")], false));

    assert!(listed.contains("2 components"), "{listed}");
    assert!(listed.contains("- Billing (`/work/billing`, 10 characters): Charges customers."), "{listed}");
}

#[test]
fn a_project_without_components_says_so() {
    assert_eq!(report(&read(vec![], false)), "`project:vega` has no components.");
}

#[test]
fn a_component_asked_for_by_name_comes_back_whole() {
    let one = report(&read(vec![component("Billing", "/work/billing", "Charges customers.", "## Purpose\nCharges.")], true));

    assert_eq!(one, "Component \"Billing\" of `project:vega`, folder `/work/billing`:\n\nCharges customers.\n\n## Purpose\nCharges.");
}
