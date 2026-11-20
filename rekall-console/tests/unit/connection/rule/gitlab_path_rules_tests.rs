use rekall_core::connection::GitlabNamespaceKind;

use super::*;

fn project(path: &str) -> GitlabNamespace {
    GitlabNamespace { kind: GitlabNamespaceKind::Project, path: path.into(), name: path.into(), visibility: None, default_branch: None, last_activity_at: None, subgroup_count: None, project_count: None }
}

#[test]
fn a_pasted_address_is_read_as_its_path() {
    assert_eq!(path_of("https://gitlab.acme.dev/platform/backend/payments-api.git"), "platform/backend/payments-api");
    assert_eq!(path_of(" /platform/backend/ "), "platform/backend");
    assert_eq!(path_of("platform"), "platform");
    assert_eq!(path_of("https://gitlab.acme.dev"), "");
}

#[test]
fn a_path_is_within_the_root_when_it_is_the_root_or_below_it_in_any_case() {
    assert!(is_within_root(Some("platform"), "platform"));
    assert!(is_within_root(Some("Platform/"), "platform/backend/ledger"));
    assert!(!is_within_root(Some("platform"), "platform-tools/app"));
    assert!(!is_within_root(Some("platform"), "other/app"));
    assert!(is_within_root(None, "anything/at/all"));
}

#[test]
fn the_parent_of_a_top_level_path_is_none() {
    assert_eq!(parent_of("platform/backend/ledger"), Some("platform/backend"));
    assert_eq!(parent_of("platform"), None);
}

#[test]
fn a_misspelling_is_matched_to_the_nearest_sibling_and_a_stranger_to_none() {
    let siblings = [project("platform/backend/payments-api"), project("platform/backend/ledger-service")];

    assert_eq!(nearest_sibling("platform/backend/paymnts-api", &siblings).as_deref(), Some("platform/backend/payments-api"));
    assert_eq!(nearest_sibling("platform/backend/zzzzzzzzzz", &siblings), None);
}
