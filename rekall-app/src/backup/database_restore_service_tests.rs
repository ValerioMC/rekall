use std::io::Write;

use super::*;

fn zip(folder: &Path, entries: &[(&str, &[u8])]) -> PathBuf {
    let path = folder.join(format!("backup-{}.zip", rekall_common::Id::random()));
    let mut writer = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
    for (name, content) in entries {
        writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
        writer.write_all(content).unwrap();
    }
    writer.finish().unwrap();
    path
}

fn sqlite(rest: &str) -> Vec<u8> {
    let mut bytes = SQLITE_HEADER.to_vec();
    bytes.extend_from_slice(rest.as_bytes());
    bytes
}

#[test]
fn the_database_file_is_taken_out_of_the_zip_under_its_own_name() {
    let folder = tempfile::tempdir().unwrap();
    let zip = zip(folder.path(), &[("rekall.db", &sqlite("the rest"))]);
    let target = folder.path().join("rekall.db.restoring");
    extract_database(&zip, "rekall.db", &target).unwrap();
    assert!(std::fs::read(&target).unwrap().starts_with(b"SQLite format 3"));
}

#[test]
fn a_backup_of_a_database_with_another_name_is_accepted_when_it_is_the_only_one_in_the_zip() {
    let folder = tempfile::tempdir().unwrap();
    let zip = zip(folder.path(), &[("other.db", &sqlite("data"))]);
    let target = folder.path().join("rekall.db.restoring");
    extract_database(&zip, "rekall.db", &target).unwrap();
    assert!(target.exists());
}

#[test]
fn a_zip_with_no_database_two_databases_or_a_file_that_is_not_sqlite_is_refused_and_leaves_nothing_behind() {
    let folder = tempfile::tempdir().unwrap();
    let target = folder.path().join("rekall.db.restoring");
    let refused = |entries: &[(&str, &[u8])]| {
        extract_database(&zip(folder.path(), entries), "rekall.db", &target).unwrap_err().message().to_string()
    };
    assert!(refused(&[("notes.md", b"hello")]).contains("no Rekall database file"));
    assert!(refused(&[("a.db", &sqlite("")), ("b.db", &sqlite(""))]).contains("more than one"));
    assert!(refused(&[("rekall.db", b"PK not a database")]).contains("not a SQLite database file"));
    assert!(refused(&[("rekall.mv.db", b"H:2 data")]).contains("--migrate-from-h2"));
    assert!(!target.exists());
}

#[test]
fn a_file_that_is_not_a_zip_at_all_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let not_zip = folder.path().join("x.zip");
    std::fs::write(&not_zip, "plain text").unwrap();
    let refused = extract_database(&not_zip, "rekall.db", &folder.path().join("out")).unwrap_err();
    assert!(matches!(refused, RekallError::IllegalArgument(_)));
}

#[test]
fn only_a_file_database_has_a_location_its_folder_its_file_and_a_backups_folder_beside_it() {
    let location = DatabaseLocation::of(Path::new("/data/rekall/rekall.db")).unwrap();
    assert_eq!(location.folder, PathBuf::from("/data/rekall"));
    assert_eq!(location.data_file(), PathBuf::from("/data/rekall/rekall.db"));
    assert_eq!(location.backups(), PathBuf::from("/data/rekall/backups"));
}
