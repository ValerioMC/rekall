use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use super::*;

/// A registry in memory over a disk where only `folders` exist, each writable; `with_database` hold one.
struct FakeFolders {
    registry: Mutex<Option<DatabaseRegistry>>,
    folders: BTreeSet<&'static str>,
    with_database: BTreeSet<&'static str>,
}

impl DatabaseFolders for FakeFolders {
    fn read(&self) -> Result<Option<DatabaseRegistry>> {
        Ok(self.registry.lock().unwrap().clone())
    }

    fn write(&self, registry: &DatabaseRegistry) -> Result<()> {
        *self.registry.lock().unwrap() = Some(registry.clone());
        Ok(())
    }

    fn inspect(&self, path: Option<&str>) -> FolderInspection {
        let path = path.unwrap_or_default();
        let exists = self.folders.contains(path);
        FolderInspection { resolved_path: path.into(), exists, is_directory: exists, writable: exists, has_database: self.with_database.contains(path) }
    }
}

#[derive(Default)]
struct CountingRestart(AtomicUsize);

impl ApplicationRestart for CountingRestart {
    fn restart(&self) -> bool {
        self.0.fetch_add(1, Ordering::SeqCst);
        true
    }
}

fn entry(id: &str, path: &str) -> DatabaseEntry {
    DatabaseEntry { id: id.into(), label: id.to_uppercase(), path: path.into(), added_at: "t".into(), last_used_at: "t".into() }
}

fn registry(active: &str, entries: Vec<DatabaseEntry>) -> Option<DatabaseRegistry> {
    Some(DatabaseRegistry { active_id: Some(active.into()), databases: entries })
}

fn service(registry: Option<DatabaseRegistry>) -> (DatabaseFolderService, Arc<FakeFolders>, Arc<CountingRestart>) {
    let folders = Arc::new(FakeFolders {
        registry: Mutex::new(registry),
        folders: BTreeSet::from(["/work", "/shared/team", "/other"]),
        with_database: BTreeSet::from(["/shared/team"]),
    });
    let restart = Arc::new(CountingRestart::default());
    (DatabaseFolderService::new(folders.clone(), restart.clone()), folders, restart)
}

#[test]
fn the_status_follows_the_active_folder() {
    let (nothing, _, _) = service(None);
    let (ready, _, _) = service(registry("a", vec![entry("a", "/work")]));
    let (unplugged, _, _) = service(registry("a", vec![entry("a", "/unplugged")]));

    assert_eq!(nothing.status().unwrap().status, SetupStatus::SetupNeeded);
    assert_eq!(ready.status().unwrap().status, SetupStatus::Ready);
    assert_eq!(unplugged.status().unwrap().status, SetupStatus::Unreachable);
}

#[test]
fn a_folder_that_is_not_there_is_refused_without_writing_or_restarting() {
    let (service, folders, restart) = service(None);

    let refused = service.add(AddDatabaseCommand { path: Some("/missing".into()), label: None });

    assert!(matches!(refused, Err(RekallError::IllegalArgument(_))));
    assert!(folders.registry.lock().unwrap().is_none());
    assert_eq!(restart.0.load(Ordering::SeqCst), 0);
}

#[test]
fn an_added_folder_becomes_active_under_its_own_name_and_restarts_onto_it() {
    let (service, folders, restart) = service(registry("a", vec![entry("a", "/work")]));

    let added = service.add(AddDatabaseCommand { path: Some("/shared/team".into()), label: Some("  ".into()) }).unwrap();

    assert!(added.opened_existing);
    assert_eq!((added.entry.label.as_str(), added.entry.active), ("team", true));
    assert_eq!(folders.registry.lock().unwrap().as_ref().unwrap().databases.len(), 2);
    assert_eq!(restart.0.load(Ordering::SeqCst), 1);
}

#[test]
fn switching_restarts_and_renaming_does_not() {
    let (service, _, restart) = service(registry("a", vec![entry("a", "/work"), entry("b", "/other")]));

    let renamed = service.rename("b", Some(" Other ")).unwrap();
    assert_eq!((renamed.label.as_str(), renamed.active), ("Other", false));
    assert_eq!(restart.0.load(Ordering::SeqCst), 0);

    assert!(service.activate("b").unwrap().active);
    assert_eq!(restart.0.load(Ordering::SeqCst), 1);
}

#[test]
fn the_active_database_cannot_be_forgotten_and_a_blank_name_is_refused() {
    let (service, _, _) = service(registry("a", vec![entry("a", "/work"), entry("b", "/other")]));

    assert!(matches!(service.forget("a"), Err(RekallError::Conflict(_))));
    assert!(matches!(service.rename("a", Some(" ")), Err(RekallError::IllegalArgument(_))));
    assert!(matches!(service.forget("missing"), Err(RekallError::NotFound(_))));
    service.forget("b").unwrap();
    assert_eq!(service.status().unwrap().databases.len(), 1);
}
