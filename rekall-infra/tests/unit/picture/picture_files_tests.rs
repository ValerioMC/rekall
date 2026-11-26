use std::path::PathBuf;

use super::*;

fn a_file(images: &Path, relative: &str) -> PathBuf {
    let file = images.join(relative);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, b"pixels").unwrap();
    file
}

fn listed(images: &Path) -> Vec<String> {
    let mut paths: Vec<String> = picture_files(images).unwrap().iter().map(|path| path.relative().to_string()).collect();
    paths.sort();
    paths
}

#[test]
fn only_files_shaped_like_pictures_are_listed() {
    let folder = tempfile::tempdir().unwrap();
    a_file(folder.path(), "acme/web/web-12/a.png");
    a_file(folder.path(), "acme/web/pages/guide/b.svg");
    for other in ["acme/web/web-12/notes.txt", "acme/web/web-12/.DS_Store", "acme/web/web-12/Shot.png", "loose.png"] {
        a_file(folder.path(), other);
    }

    assert_eq!(listed(folder.path()), ["acme/web/pages/guide/b.svg", "acme/web/web-12/a.png"]);
}

#[cfg(unix)]
#[test]
fn a_link_is_not_followed() {
    let folder = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    a_file(outside.path(), "web/web-12/secret.png");
    std::os::unix::fs::symlink(outside.path(), folder.path().join("acme")).unwrap();

    assert!(listed(folder.path()).is_empty());
}

#[test]
fn a_folder_that_is_not_there_holds_no_pictures() {
    let folder = tempfile::tempdir().unwrap();

    assert!(picture_files(&folder.path().join("images")).unwrap().is_empty());
}
