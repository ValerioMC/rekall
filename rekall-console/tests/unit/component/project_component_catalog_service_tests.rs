use super::*;

#[test]
fn a_folder_is_stored_without_its_trailing_separator() {
    assert_eq!(folder_of(Some("  /work/api/ ")).unwrap(), "/work/api");
    assert_eq!(folder_of(Some("services\\billing\\")).unwrap(), "services\\billing");
}

#[test]
fn the_root_folder_is_kept_whole() {
    assert_eq!(folder_of(Some("/")).unwrap(), "/");
}

#[test]
fn a_missing_or_blank_folder_is_refused_as_a_client_mistake() {
    assert!(folder_of(None).is_err());
    assert!(folder_of(Some("   ")).is_err());
}

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
        description: String::new(),
        position: 0,
        created_at: now,
        updated_at: now,
    };
    let components = [named("Billing"), named("billing-2")];

    assert_eq!(first_free_name("orders", &components), "orders");
    assert_eq!(first_free_name("billing", &components), "billing-3");
}
