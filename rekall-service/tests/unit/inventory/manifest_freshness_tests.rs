use rekall_common::Id;
use rekall_store::Ecosystem;

use super::*;

fn row(manifest: &str) -> project_dependency::Model {
    project_dependency::Model {
        id: Id::random(),
        project_id: Id::random(),
        ecosystem: Ecosystem::Npm,
        name: "left-pad".into(),
        version: "1.0.0".into(),
        manifest: manifest.into(),
        created_at: Instant::now(),
    }
}

#[test]
fn a_manifest_written_after_the_inventory_or_gone_since_has_changed() {
    let folder = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(folder.path().join("web")).unwrap();
    std::fs::write(folder.path().join("web/package-lock.json"), "{}").unwrap();
    std::fs::write(folder.path().join("Cargo.lock"), "").unwrap();
    let rows = [row("web/package-lock.json"), row("web/package-lock.json"), row("Cargo.lock"), row("gone/yarn.lock")];
    let folder = folder.path().to_string_lossy().into_owned();

    let recorded_before = changed_since(Some(&folder), &rows, Instant::now().plus_minutes(-60));
    let recorded_after = changed_since(Some(&folder), &rows, Instant::now().plus_minutes(60));

    assert_eq!(recorded_before, vec!["Cargo.lock", "gone/yarn.lock", "web/package-lock.json"]);
    assert_eq!(recorded_after, vec!["gone/yarn.lock"], "a manifest that is gone has changed whenever it was recorded");
}

#[test]
fn a_project_without_a_folder_has_nothing_to_compare() {
    assert!(changed_since(None, &[row("Cargo.lock")], Instant::now()).is_empty());
    assert!(changed_since(Some("  "), &[row("Cargo.lock")], Instant::now()).is_empty());
}
