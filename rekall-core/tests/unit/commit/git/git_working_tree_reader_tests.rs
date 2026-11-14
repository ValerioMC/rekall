use std::path::Path;

use super::*;

async fn git(folder: &Path, arguments: &[&str]) {
    let result = run(folder, arguments, None, READ_TIMEOUT).await.expect("git runs");
    assert!(result.ok(), "git {arguments:?}: {}", result.stderr);
}

async fn repository_with_one_commit() -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    git(folder.path(), &["init", "-q"]).await;
    git(folder.path(), &["config", "user.email", "test@example.com"]).await;
    git(folder.path(), &["config", "user.name", "Test"]).await;
    std::fs::write(folder.path().join("kept.txt"), "one\ntwo\n").unwrap();
    git(folder.path(), &["add", "."]).await;
    git(folder.path(), &["commit", "-q", "-m", "first"]).await;
    folder
}

#[tokio::test]
async fn a_clean_tree_has_an_empty_patch() {
    let folder = repository_with_one_commit().await;

    assert_eq!(GitWorkingTreeReader.patch(folder.path()).await.unwrap(), "");
}

#[tokio::test]
async fn an_edited_file_and_an_untracked_one_both_appear_and_nothing_is_staged() {
    let folder = repository_with_one_commit().await;
    std::fs::write(folder.path().join("kept.txt"), "one\nTWO\n").unwrap();
    std::fs::write(folder.path().join("fresh.txt"), "hello\n").unwrap();

    let patch = GitWorkingTreeReader.patch(folder.path()).await.unwrap();

    assert!(patch.contains("diff --git a/kept.txt b/kept.txt"));
    assert!(patch.contains("+TWO"));
    assert!(patch.contains("new file mode"));
    assert!(patch.contains("+hello"));
    let staged = run(folder.path(), &["diff", "--cached", "--name-only"], None, READ_TIMEOUT).await.unwrap();
    assert_eq!(staged.output(), "");
}

#[tokio::test]
async fn a_folder_that_is_no_repository_is_refused() {
    let folder = tempfile::tempdir().unwrap();

    assert!(GitWorkingTreeReader.patch(folder.path()).await.is_err());
}
