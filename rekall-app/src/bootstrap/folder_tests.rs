use super::*;

fn home() -> PathBuf {
    PathBuf::from("/home/someone")
}

#[test]
fn a_folder_that_does_not_exist_is_reported_as_such() {
    let temp = tempfile::tempdir().unwrap();
    let result = inspect(Some(temp.path().join("nowhere").to_str().unwrap()), &home());
    assert!(!result.exists);
    assert!(!result.usable());
}

#[test]
fn an_empty_existing_folder_is_usable_but_has_no_database_yet() {
    let temp = tempfile::tempdir().unwrap();
    let result = inspect(temp.path().to_str(), &home());
    assert!(result.usable());
    assert!(!result.has_database);
}

#[test]
fn a_folder_with_an_mv_db_file_or_a_sqlite_one_is_reported_as_already_having_a_database() {
    let legacy = tempfile::tempdir().unwrap();
    std::fs::File::create(legacy.path().join("rekall.mv.db")).unwrap();
    assert!(inspect(legacy.path().to_str(), &home()).has_database);

    let current = tempfile::tempdir().unwrap();
    std::fs::File::create(current.path().join("rekall.db")).unwrap();
    assert!(inspect(current.path().to_str(), &home()).has_database);
}

#[test]
fn blank_input_is_never_usable() {
    assert!(!inspect(Some("   "), &home()).usable());
    assert!(!inspect(None, &home()).usable());
}

#[test]
fn a_file_rather_than_a_folder_is_not_usable() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("not-a-folder");
    std::fs::File::create(&file).unwrap();
    let result = inspect(file.to_str(), &home());
    assert!(result.exists);
    assert!(!result.is_directory);
    assert!(!result.usable());
}

#[test]
fn a_leading_tilde_expands_to_the_home_directory() {
    assert_eq!(inspect(Some("~"), &home()).resolved_path, "/home/someone");
    assert_eq!(inspect(Some("~/a/../b"), &home()).resolved_path, "/home/someone/b");
}
