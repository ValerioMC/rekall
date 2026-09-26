use super::*;
use crate::config::Properties;

fn config(home: &Path, working_dir: &Path) -> AppConfig {
    let mut config = AppConfig::from_sources(&Properties::of(&[("rekall.home", home.to_str().unwrap())]));
    config.working_dir = working_dir.to_path_buf();
    config
}

#[test]
fn nothing_registered_and_no_legacy_folder_is_the_setup_screen_on_memory() {
    let home = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let resolved = resolve(&config(home.path(), work.path())).unwrap();
    assert_eq!(resolved, Resolved { database: DatabaseOverride::Memory, status: SetupStatus::SetupNeeded });
}

#[test]
fn a_legacy_data_folder_is_adopted_as_local_and_opened() {
    let home = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(work.path().join("data")).unwrap();
    std::fs::File::create(work.path().join("data/rekall.mv.db")).unwrap();

    let resolved = resolve(&config(home.path(), work.path())).unwrap();
    assert_eq!(resolved.status, SetupStatus::Ready);
    assert_eq!(resolved.database, DatabaseOverride::File(work.path().join("data/rekall.db")));
    let registry = DatabaseRegistryStore::new(home.path()).read().unwrap().unwrap();
    assert_eq!(registry.active().unwrap().label, "Local");
}

#[test]
fn an_active_folder_that_is_gone_is_unreachable_and_runs_on_memory() {
    let home = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let entry = DatabaseEntry {
        id: "a".into(),
        label: "Drive".into(),
        path: "/definitely/not/here".into(),
        added_at: "t".into(),
        last_used_at: "t".into(),
    };
    DatabaseRegistryStore::new(home.path())
        .write(&DatabaseRegistry { active_id: Some("a".into()), databases: vec![entry] })
        .unwrap();
    let resolved = resolve(&config(home.path(), work.path())).unwrap();
    assert_eq!(resolved, Resolved { database: DatabaseOverride::Memory, status: SetupStatus::Unreachable });
}
