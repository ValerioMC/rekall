use std::fs;

use super::*;

fn repository(path: &Path) {
    fs::create_dir_all(path.join(".git")).unwrap();
}

#[test]
fn repositories_are_found_at_any_level_and_plain_folders_are_walked_through() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path();
    repository(&root.join("a"));
    repository(&root.join("b/a1"));
    repository(&root.join("b/a2"));
    fs::create_dir_all(root.join("c/empty")).unwrap();
    repository(&root.join("d"));

    let found: Vec<PathBuf> = repositories_under(root).into_iter().map(|path| path.strip_prefix(root).unwrap().to_path_buf()).collect();

    assert_eq!(found, ["a", "b/a1", "b/a2", "d"].map(PathBuf::from));
}

#[test]
fn a_repository_is_not_searched_for_more_inside_it() {
    let root = tempfile::tempdir().unwrap();
    repository(root.path());
    repository(&root.path().join("vendor/lib"));

    assert_eq!(repositories_under(root.path()), [root.path().to_path_buf()]);
}

#[test]
fn hidden_folders_dependency_caches_and_deep_folders_are_passed_over() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path();
    repository(&root.join(".cache/clone"));
    repository(&root.join("node_modules/pkg"));
    repository(&root.join("1/2/3/4/5"));
    repository(&root.join("1/2/3/4b"));

    assert_eq!(repositories_under(root), [root.join("1/2/3/4b")]);
}

#[test]
fn a_worktree_whose_git_is_a_file_counts_as_a_repository() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("wt")).unwrap();
    fs::write(root.path().join("wt/.git"), "gitdir: /elsewhere").unwrap();

    assert_eq!(repositories_under(root.path()), [root.path().join("wt")]);
}
