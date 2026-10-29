use super::*;

fn scratch() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("rekall-cleanup-{}", uuid::Uuid::new_v4()))
}

#[test]
fn a_file_is_removed() {
    let file = scratch();
    std::fs::write(&file, "leftover").unwrap();
    remove_file(&file);
    assert!(!file.exists());
}

#[test]
fn a_file_already_gone_is_not_a_failure() {
    remove_file(&scratch());
}

#[test]
fn a_folder_is_removed_with_what_it_holds() {
    let folder = scratch();
    std::fs::create_dir_all(folder.join("inner")).unwrap();
    std::fs::write(folder.join("inner/file"), "x").unwrap();
    remove_dir_all(&folder);
    assert!(!folder.exists());
}

#[cfg(unix)]
#[test]
fn a_running_child_is_killed_and_reaped() {
    let mut child = std::process::Command::new("sleep").arg("30").spawn().unwrap();
    kill(&mut child);
    assert!(child.try_wait().unwrap().is_some());
}

#[cfg(unix)]
#[test]
fn a_child_that_already_exited_is_reaped_without_failing() {
    let mut child = std::process::Command::new("true").spawn().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    kill(&mut child);
    assert!(child.try_wait().unwrap().is_some());
}
