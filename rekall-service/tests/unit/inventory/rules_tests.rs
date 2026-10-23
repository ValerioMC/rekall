use super::*;

fn declared(ecosystem: &str, name: &str, version: &str, manifest: &str) -> DeclaredDependency {
    DeclaredDependency { ecosystem: Some(ecosystem.into()), name: Some(name.into()), version: Some(version.into()), manifest: Some(manifest.into()) }
}

#[test]
fn an_ecosystem_is_read_by_its_advisory_name_ignoring_case() {
    assert_eq!(ecosystem(Some("NPM")).unwrap(), Ecosystem::Npm);
    assert_eq!(ecosystem(Some(" rust ")).unwrap(), Ecosystem::Rust);
    let message = ecosystem(Some("cargo")).unwrap_err().message().to_string();
    assert!(message.contains("'cargo' is not an ecosystem") && message.contains("rust"), "{message}");
}

#[test]
fn a_version_is_the_exact_one_resolved_never_a_range() {
    assert_eq!(version(Some("4.17.21")).unwrap(), "4.17.21");
    assert_eq!(version(Some("v1.2.3")).unwrap(), "1.2.3");
    assert_eq!(version(Some("very-1")).unwrap(), "very-1");
    for range in ["^4.17.0", "~1.2", ">=2.0", "1.x || 2.x", "*", "1.0, 2.0"] {
        assert!(version(Some(range)).unwrap_err().message().contains("is a range"), "{range}");
    }
}

#[test]
fn every_entry_is_stored_once_and_the_first_manifest_is_kept() {
    let project = Id::random();
    let rows = dependencies(
        project,
        &[declared("npm", "lodash", "4.17.21", "package-lock.json"), declared("NPM", "lodash", "4.17.21", "web/package-lock.json"), declared("npm", "lodash", "4.17.20", "web/package-lock.json")],
        Instant::now(),
    )
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].manifest, "package-lock.json");
    assert!(rows.iter().all(|row| row.project_id == project));
}

#[test]
fn a_refusal_names_the_entry_by_its_position() {
    let message = dependencies(Id::random(), &[declared("npm", "a", "1.0.0", "package.json"), declared("npm", "", "1.0.0", "package.json")], Instant::now())
        .unwrap_err()
        .message()
        .to_string();
    assert!(message.starts_with("dependencies[1]: a name is required"), "{message}");
}

#[test]
fn a_manifest_stays_inside_the_project_folder() {
    let at = |path: &str| dependencies(Id::random(), &[declared("pip", "django", "4.2.0", path)], Instant::now());
    assert_eq!(at("./backend\\requirements.txt").unwrap()[0].manifest, "backend/requirements.txt");
    assert!(at("/etc/requirements.txt").unwrap_err().message().contains("inside the project's folder"));
    assert!(at("../other/requirements.txt").unwrap_err().message().contains("inside the project's folder"));
}

#[test]
fn an_empty_inventory_is_refused() {
    assert!(dependencies(Id::random(), &[], Instant::now()).unwrap_err().message().contains("at least one dependency"));
}
