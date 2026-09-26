//! The git wrapper against the real `git` binary: mocking a thin process wrapper would test nothing.

mod support;

use std::fs;

use rekall_service::commit::{GitCommitter, GitLogReader, GitRepositoryInspector, PendingChange, PendingChangeKind, RepositoryStatus};
use support::{commit, git, init_with_identity};

#[tokio::test]
async fn lists_untracked_modified_and_deleted_files_as_pending_changes() {
    let repo = tempfile::tempdir().unwrap();
    let repo = repo.path();
    init_with_identity(repo);
    fs::write(repo.join("kept.txt"), "one").unwrap();
    fs::write(repo.join("gone.txt"), "two").unwrap();
    git(repo, &["add", "-A"]);
    commit(repo, "seed");
    fs::write(repo.join("kept.txt"), "one more").unwrap();
    fs::remove_file(repo.join("gone.txt")).unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(repo.join("src/new.txt"), "three").unwrap();

    let mut changes = GitCommitter.pending_changes(repo).await.unwrap();
    changes.sort_by(|a, b| a.path.cmp(&b.path));
    assert_eq!(
        changes,
        vec![
            PendingChange::new(PendingChangeKind::Deleted, "gone.txt"),
            PendingChange::new(PendingChangeKind::Modified, "kept.txt"),
            PendingChange::new(PendingChangeKind::Added, "src/new.txt"),
        ]
    );
}

#[tokio::test]
async fn a_clean_tree_has_no_pending_change() {
    let repo = tempfile::tempdir().unwrap();
    let repo = repo.path();
    init_with_identity(repo);
    fs::write(repo.join("a.txt"), "a").unwrap();
    git(repo, &["add", "-A"]);
    commit(repo, "seed");
    assert!(GitCommitter.pending_changes(repo).await.unwrap().is_empty());
}

#[tokio::test]
async fn commits_everything_with_the_multi_line_message_and_returns_the_new_tip() {
    let repo = tempfile::tempdir().unwrap();
    let repo = repo.path();
    init_with_identity(repo);
    fs::write(repo.join("a.txt"), "a").unwrap();
    fs::write(repo.join("b.txt"), "b").unwrap();

    let hash = GitCommitter.commit_all(repo, "feat: the subject\n\nproject:vega task:x\n2 files: 2 added\n").await.unwrap();

    assert_eq!(hash, git(repo, &["rev-parse", "HEAD"]));
    assert_eq!(git(repo, &["log", "-1", "--format=%s"]), "feat: the subject");
    assert!(git(repo, &["log", "-1", "--format=%b"]).contains("project:vega task:x"));
    assert!(git(repo, &["status", "--porcelain"]).is_empty());
}

#[tokio::test]
async fn a_folder_that_is_not_a_repository_is_refused_with_gits_reason_and_nothing_is_written() {
    let parent = tempfile::tempdir().unwrap();
    // A folder outside any repository, so git cannot find one above it either.
    let folder = parent.path();
    fs::write(folder.join("a.txt"), "a").unwrap();
    let pending = GitCommitter.pending_changes(folder).await.unwrap_err();
    assert!(pending.message().contains("Could not read the working tree"), "{pending}");
    let staged = GitCommitter.commit_all(folder, "feat: nowhere").await.unwrap_err();
    assert!(staged.message().contains("Could not stage"), "{staged}");
    assert!(!folder.join(".git").exists());
}

#[tokio::test]
async fn reads_the_hash_subject_and_diff_of_the_tip_commit() {
    let repo = tempfile::tempdir().unwrap();
    let repo = repo.path();
    git(repo, &["init", "-q"]);
    fs::write(repo.join("README.md"), "hello").unwrap();
    git(repo, &["add", "README.md"]);
    commit(repo, "Add the readme");

    let head = GitLogReader.head(repo).await.unwrap();
    assert_eq!(head.hash, git(repo, &["rev-parse", "HEAD"]));
    assert_eq!(head.subject, "Add the readme");
    let diff = head.diff.unwrap();
    assert!(diff.contains("+hello") && diff.contains("README.md"));
}

#[tokio::test]
async fn the_diff_of_a_second_commit_is_only_what_it_changed() {
    let repo = tempfile::tempdir().unwrap();
    let repo = repo.path();
    git(repo, &["init", "-q"]);
    fs::write(repo.join("a.txt"), "one\n").unwrap();
    git(repo, &["add", "a.txt"]);
    commit(repo, "Add a.txt");
    fs::write(repo.join("a.txt"), "one\ntwo\n").unwrap();
    git(repo, &["add", "a.txt"]);
    commit(repo, "Append a second line");

    let diff = GitLogReader.head(repo).await.unwrap().diff.unwrap();
    assert!(diff.contains("+two") && !diff.contains("+one"));
}

#[tokio::test]
async fn only_the_subject_line_is_kept_even_when_the_commit_has_a_body() {
    let repo = tempfile::tempdir().unwrap();
    let repo = repo.path();
    git(repo, &["init", "-q"]);
    fs::write(repo.join("a.txt"), "a").unwrap();
    git(repo, &["add", "a.txt"]);
    commit(repo, "Short subject\n\nA much longer body explaining why.");
    assert_eq!(GitLogReader.head(repo).await.unwrap().subject, "Short subject");
}

#[tokio::test]
async fn a_repository_with_no_commits_a_plain_folder_and_a_missing_one_are_refused() {
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q"]);
    assert!(GitLogReader.head(repo.path()).await.unwrap_err().is_illegal_argument());

    let plain = tempfile::tempdir().unwrap();
    assert!(GitLogReader.head(plain.path()).await.unwrap_err().is_illegal_argument());

    let missing = plain.path().join("does-not-exist");
    let error = GitLogReader.head(&missing).await.unwrap_err();
    assert!(error.message().contains("not there"), "{error}");
}

#[tokio::test]
async fn a_past_commit_is_read_by_its_full_or_abbreviated_hash() {
    let repo = tempfile::tempdir().unwrap();
    let repo = repo.path();
    git(repo, &["init", "-q"]);
    fs::write(repo.join("a.txt"), "one\n").unwrap();
    git(repo, &["add", "a.txt"]);
    commit(repo, "Add a.txt");
    let first = git(repo, &["rev-parse", "HEAD"]);
    fs::write(repo.join("a.txt"), "one\ntwo\n").unwrap();
    git(repo, &["add", "a.txt"]);
    commit(repo, "Append a second line");

    let past = GitLogReader.commit(repo, &first).await.unwrap();
    assert_eq!(past.hash, first);
    assert_eq!(past.subject, "Add a.txt");
    let diff = past.diff.unwrap();
    assert!(diff.contains("+one") && !diff.contains("+two"));

    assert_eq!(GitLogReader.commit(repo, &first[..7]).await.unwrap().hash, first);
}

#[tokio::test]
async fn a_hash_that_names_no_commit_or_is_not_hex_is_refused() {
    let repo = tempfile::tempdir().unwrap();
    let repo = repo.path();
    git(repo, &["init", "-q"]);
    fs::write(repo.join("a.txt"), "a").unwrap();
    git(repo, &["add", "a.txt"]);
    commit(repo, "Add a.txt");

    assert!(GitLogReader.commit(repo, "deadbeef").await.unwrap_err().message().contains("deadbeef"));
    for reference in ["--output=/tmp/x", "main"] {
        let error = GitLogReader.commit(repo, reference).await.unwrap_err();
        assert!(error.message().contains("not a commit hash"), "{error}");
    }
}

#[tokio::test]
async fn the_recent_log_comes_back_newest_first_and_stops_at_the_limit() {
    let repo = tempfile::tempdir().unwrap();
    let repo = repo.path();
    git(repo, &["init", "-q"]);
    for i in 1..=4 {
        fs::write(repo.join("a.txt"), format!("line {i}\n")).unwrap();
        git(repo, &["add", "a.txt"]);
        commit(repo, &format!("Commit {i}"));
    }
    let log = GitLogReader.recent(repo, 10).await.unwrap();
    let subjects: Vec<&str> = log.iter().map(|e| e.subject.as_str()).collect();
    assert_eq!(subjects, ["Commit 4", "Commit 3", "Commit 2", "Commit 1"]);
    assert_eq!(log[0].hash, git(repo, &["rev-parse", "HEAD"]));
    let limited: Vec<String> = GitLogReader.recent(repo, 2).await.unwrap().into_iter().map(|e| e.subject).collect();
    assert_eq!(limited, ["Commit 4", "Commit 3"]);
}

#[tokio::test]
async fn the_recent_log_of_a_repository_with_no_commits_yet_is_refused() {
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q"]);
    assert!(GitLogReader.recent(repo.path(), 10).await.unwrap_err().is_illegal_argument());
}

#[tokio::test]
async fn the_inspector_tells_unset_missing_plain_and_repository_apart() {
    assert_eq!(GitRepositoryInspector.inspect(None).await, RepositoryStatus::unset());
    assert_eq!(GitRepositoryInspector.inspect(Some("   ")).await, RepositoryStatus::unset());

    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nope").to_string_lossy().into_owned();
    assert_eq!(GitRepositoryInspector.inspect(Some(&missing)).await, RepositoryStatus::missing(&missing));
    assert!(!GitRepositoryInspector.is_repository(Some(&missing)).await);

    let plain = GitRepositoryInspector.inspect(Some(&dir.path().to_string_lossy())).await;
    assert!(plain.exists && !plain.repository && !plain.can_commit());

    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q", "-b", "main"]);
    git(repo.path(), &["config", "user.name", "Test Person"]);
    git(repo.path(), &["config", "user.email", "test@example.com"]);
    fs::write(repo.path().join("a.txt"), "a").unwrap();
    git(repo.path(), &["add", "-A"]);
    commit(repo.path(), "seed");
    let path = repo.path().to_string_lossy().into_owned();
    let status = GitRepositoryInspector.inspect(Some(&format!(" {path} "))).await;
    assert_eq!(status.folder.as_deref(), Some(path.as_str()));
    assert!(status.repository);
    assert_eq!(status.branch.as_deref(), Some("main"));
    assert_eq!(status.user_name.as_deref(), Some("Test Person"));
    assert_eq!(status.user_email.as_deref(), Some("test@example.com"));
    assert!(status.can_commit());
}

#[tokio::test]
async fn a_repository_with_no_commit_yet_still_names_its_branch() {
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q", "-b", "trunk"]);
    let status = GitRepositoryInspector.inspect(Some(&repo.path().to_string_lossy())).await;
    assert_eq!(status.branch.as_deref(), Some("trunk"));
}
