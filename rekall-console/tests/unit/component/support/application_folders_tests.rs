use std::fs;

use super::*;

fn application(path: &Path, manifest: &str) {
    fs::create_dir_all(path).unwrap();
    fs::write(path.join(manifest), "").unwrap();
}

fn relative(root: &Path, found: Vec<PathBuf>) -> Vec<PathBuf> {
    found.into_iter().map(|path| path.strip_prefix(root).unwrap().to_path_buf()).collect()
}

#[test]
fn a_repository_of_several_applications_is_one_component_per_application() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path();
    application(&root.join("orders"), "pom.xml");
    application(&root.join("billing"), "package.json");
    application(&root.join("services/ledger"), "Cargo.toml");
    fs::create_dir_all(root.join("docs")).unwrap();

    assert_eq!(relative(root, applications_in(root)), ["billing", "orders", "services/ledger"].map(PathBuf::from));
}

#[test]
fn a_repository_with_a_manifest_at_its_root_stays_one_component() {
    let root = tempfile::tempdir().unwrap();
    application(root.path(), "Cargo.toml");
    application(&root.path().join("crates/a"), "Cargo.toml");
    application(&root.path().join("crates/b"), "Cargo.toml");

    assert_eq!(applications_in(root.path()), [root.path().to_path_buf()]);
}

#[test]
fn a_repository_with_one_application_or_none_stays_one_component() {
    let one = tempfile::tempdir().unwrap();
    application(&one.path().join("api"), "go.mod");
    let none = tempfile::tempdir().unwrap();
    fs::create_dir_all(none.path().join("docs")).unwrap();

    assert_eq!(applications_in(one.path()), [one.path().to_path_buf()]);
    assert_eq!(applications_in(none.path()), [none.path().to_path_buf()]);
}

#[test]
fn an_application_is_not_searched_for_more_inside_it() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path();
    application(&root.join("a"), "Cargo.toml");
    application(&root.join("a/inner"), "Cargo.toml");
    application(&root.join("b"), "package.json");

    assert_eq!(relative(root, applications_in(root)), ["a", "b"].map(PathBuf::from));
}

#[test]
fn build_output_dependency_and_hidden_folders_are_not_applications() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path();
    application(&root.join("api"), "package.json");
    application(&root.join("web"), "package.json");
    application(&root.join("target/package"), "Cargo.toml");
    application(&root.join("vendor/lib"), "go.mod");
    application(&root.join("node_modules/pkg"), "package.json");
    application(&root.join(".cache/tool"), "package.json");

    assert_eq!(relative(root, applications_in(root)), ["api", "web"].map(PathBuf::from));
}
