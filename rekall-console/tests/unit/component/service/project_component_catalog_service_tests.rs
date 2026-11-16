use super::*;

#[test]
fn a_component_starts_out_named_after_its_folder() {
    assert_eq!(folder_name("/work/rekall/rekall-ui"), "rekall-ui");
    assert_eq!(folder_name("C:\\work\\billing"), "billing");
    assert_eq!(folder_name("/"), "/");
}

#[test]
fn a_name_another_component_holds_takes_the_first_free_number_ignoring_case() {
    let now = rekall_kernel::Instant::now();
    let named = |name: &str| project_component::Model {
        id: Id::random(),
        project_id: Id::random(),
        name: name.into(),
        folder: format!("/work/{name}"),
        summary: String::new(),
        description: String::new(),
        position: 0,
        created_at: now,
        updated_at: now,
    };
    let components = [named("Billing"), named("billing-2")];

    assert_eq!(first_free_name("orders", &components), "orders");
    assert_eq!(first_free_name("billing", &components), "billing-3");
}
