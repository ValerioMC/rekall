//! The `git` process wrapper: one invocation in a folder, its exit code and both streams, and
//! the three components built on it. A hung or unrunnable git is reported, never waited on
//! forever; every failure is an `IllegalArgument` the caller turns into its own words.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use rekall_common::{jstr, Instant, RekallError, Result};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// The folder as `Path.of(folder)` printed it: no trailing separator, no doubled ones.
pub fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    let mut out = String::with_capacity(text.len());
    let mut previous_slash = false;
    for c in text.chars() {
        if c == '/' {
            if previous_slash {
                continue;
            }
            previous_slash = true;
        } else {
            previous_slash = false;
        }
        out.push(c);
    }
    while out.len() > 1 && out.ends_with('/') {
        out.pop();
    }
    out
}

/// What git answered. `stdout` is untouched (a porcelain status line starts with a meaningful
/// space); `output()` and `error()` are the stripped reads.
#[derive(Clone, Debug)]
pub struct GitResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl GitResult {
    pub fn ok(&self) -> bool {
        self.exit_code == 0
    }

    pub fn output(&self) -> &str {
        jstr::strip(&self.stdout)
    }

    pub fn error(&self) -> &str {
        jstr::strip(&self.stderr)
    }
}

/// `GitCommand.run`: `git -C <folder> <arguments>`, with `stdin` fed to it when given (how a
/// multi-line commit message travels), refused after `timeout`.
pub async fn run(folder: &Path, arguments: &[&str], stdin: Option<&str>, timeout: Duration) -> Result<GitResult> {
    let shown = display_path(folder);
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(folder)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command
        .spawn()
        .map_err(|e| RekallError::illegal(format!("Could not run git in {shown}: {e}")))?;
    if let Some(mut input) = child.stdin.take() {
        if let Some(text) = stdin {
            input
                .write_all(text.as_bytes())
                .await
                .map_err(|e| RekallError::illegal(format!("Could not read git's output in {shown}: {e}")))?;
        }
        drop(input);
    }
    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Err(_) => Err(RekallError::illegal(format!("git took too long to answer in {shown}."))),
        Ok(Err(e)) => Err(RekallError::illegal(format!("Could not read git's output in {shown}: {e}"))),
        Ok(Ok(output)) => Ok(GitResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }),
    }
}

const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);
const READ_TIMEOUT: Duration = Duration::from_secs(5);

// ---------------------------------------------------------------------------- pending changes

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// One line of `git status --porcelain`: what the working tree holds that HEAD does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingChange {
    pub kind: PendingChangeKind,
    pub path: String,
}

impl PendingChange {
    pub fn new(kind: PendingChangeKind, path: &str) -> Self {
        Self { kind, path: path.to_string() }
    }

    /// Reads a porcelain v1 line (`XY path`, or `XY old -> new` for a rename). An untracked file
    /// counts as added: `git add -A` is about to stage it.
    pub fn parse(line: &str) -> Result<Self> {
        if jstr::len(line) < 4 {
            return Err(RekallError::illegal(format!("Unexpected git status line: '{line}'")));
        }
        let mut chars = line.chars();
        let index = chars.next().unwrap_or(' ');
        let worktree = chars.next().unwrap_or(' ');
        let path = jstr::suffix(line, 3);
        if index == 'R' || worktree == 'R' {
            let path = match path.find(" -> ") {
                Some(arrow) => &path[arrow + 4..],
                None => path,
            };
            return Ok(Self::new(PendingChangeKind::Renamed, path));
        }
        if index == '?' || index == 'A' || worktree == 'A' {
            return Ok(Self::new(PendingChangeKind::Added, path));
        }
        if index == 'D' || worktree == 'D' {
            return Ok(Self::new(PendingChangeKind::Deleted, path));
        }
        Ok(Self::new(PendingChangeKind::Modified, path))
    }
}

// ---------------------------------------------------------------------------- committer

/// The one place that writes to a repository: stages everything and commits it, as whoever git
/// is configured to be in that folder.
#[derive(Clone, Copy, Debug, Default)]
pub struct GitCommitter;

impl GitCommitter {
    /// Every change `git add -A` would pick up, untracked files listed one by one.
    pub async fn pending_changes(&self, folder: &Path) -> Result<Vec<PendingChange>> {
        let status = run(folder, &["status", "--porcelain", "-uall"], None, COMMAND_TIMEOUT).await?;
        if !status.ok() {
            return Err(RekallError::illegal(format!(
                "Could not read the working tree in {}: {}",
                display_path(folder),
                status.error()
            )));
        }
        status
            .stdout
            .split('\n')
            .filter(|line| !jstr::is_blank(line))
            .map(PendingChange::parse)
            .collect()
    }

    /// Stages everything, commits it with `message`, and hands back the new tip's hash.
    pub async fn commit_all(&self, folder: &Path, message: &str) -> Result<String> {
        let shown = display_path(folder);
        let staged = run(folder, &["add", "-A"], None, COMMAND_TIMEOUT).await?;
        if !staged.ok() {
            return Err(RekallError::illegal(format!("Could not stage the changes in {shown}: {}", staged.error())));
        }
        let committed = run(folder, &["commit", "-q", "-F", "-"], Some(message), COMMAND_TIMEOUT).await?;
        if !committed.ok() {
            let reason = if jstr::is_blank(committed.error()) { committed.output() } else { committed.error() };
            return Err(RekallError::illegal(format!("git commit failed in {shown}: {reason}")));
        }
        let tip = run(folder, &["rev-parse", "HEAD"], None, COMMAND_TIMEOUT).await?;
        if !tip.ok() || jstr::is_blank(tip.output()) {
            return Err(RekallError::illegal(format!("Committed, but could not read the new tip in {shown}.")));
        }
        Ok(tip.output().to_string())
    }
}

// ---------------------------------------------------------------------------- log reader

/// ASCII unit separator: not going to show up in a commit subject by accident.
const FIELD_SEPARATOR: char = '\u{1F}';

/// A commit read with its patch, ready to be logged against a task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoggedCommit {
    pub hash: String,
    pub subject: String,
    pub diff: Option<String>,
}

/// One line of the recent log: enough to recognise a commit, without its patch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    pub hash: String,
    pub subject: String,
    pub committed_at: Instant,
}

/// Reads commits from a repository on disk: the tip, one named by its hash, or the recent log.
/// Never writes.
#[derive(Clone, Copy, Debug, Default)]
pub struct GitLogReader;

impl GitLogReader {
    pub async fn head(&self, folder: &Path) -> Result<LoggedCommit> {
        self.commit(folder, "HEAD").await
    }

    /// The commit a hash names. The hash is validated as hex before it reaches git, so a pasted
    /// string cannot become a flag; a prefix that matches no commit, or more than one, is refused
    /// with git's own reason.
    pub async fn commit(&self, folder: &Path, reference: &str) -> Result<LoggedCommit> {
        static HASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-fA-F]{4,40}$").unwrap());
        require_repository(folder)?;
        let tip = reference == "HEAD";
        if !tip && !HASH.is_match(reference) {
            return Err(RekallError::illegal(format!(
                "'{reference}' is not a commit hash: paste 4 to 40 hex characters from git log."
            )));
        }
        let shown = display_path(folder);
        let format = format!("--format=%H{FIELD_SEPARATOR}%s");
        let failure = if tip {
            format!("Could not read the last commit in {shown}")
        } else {
            format!("No commit '{reference}' in {shown}.")
        };
        let output = read(folder, &["log", "-1", &format, reference, "--"], &failure, tip).await?;
        let Some(separator) = output.find(FIELD_SEPARATOR) else {
            return Err(RekallError::illegal(format!("Unexpected git output in {shown}.")));
        };
        let hash = output[..separator].to_string();
        let subject = output[separator + FIELD_SEPARATOR.len_utf8()..].to_string();
        let diff = diff_of(folder, &hash).await;
        Ok(LoggedCommit { hash, subject, diff })
    }

    /// The newest `limit` commits, newest first.
    pub async fn recent(&self, folder: &Path, limit: usize) -> Result<Vec<LogEntry>> {
        require_repository(folder)?;
        let shown = display_path(folder);
        let format = format!("--format=%H{FIELD_SEPARATOR}%s{FIELD_SEPARATOR}%cI");
        let limit = limit.to_string();
        let output = read(folder, &["log", "-n", &limit, &format, "--"], &format!("Could not read the log in {shown}"), true)
            .await?;
        let mut entries = Vec::new();
        for line in output.split('\n') {
            let fields: Vec<&str> = line.split(FIELD_SEPARATOR).collect();
            if fields.len() != 3 {
                return Err(RekallError::illegal(format!("Unexpected git output in {shown}.")));
            }
            let committed_at = Instant::parse(fields[2])
                .map_err(|_| RekallError::illegal(format!("Unexpected commit date from git: {}", fields[2])))?;
            entries.push(LogEntry { hash: fields[0].to_string(), subject: fields[1].to_string(), committed_at });
        }
        Ok(entries)
    }
}

fn require_repository(folder: &Path) -> Result<()> {
    if !folder.is_dir() {
        return Err(RekallError::illegal(format!(
            "This project's folder ({}) is not there.",
            display_path(folder)
        )));
    }
    Ok(())
}

/// One read-only git command's stripped stdout. A non-zero exit or an empty answer becomes
/// `failure`, with git's own stderr appended when `with_reason` is set.
async fn read(folder: &Path, arguments: &[&str], failure: &str, with_reason: bool) -> Result<String> {
    let result = run(folder, arguments, None, READ_TIMEOUT).await?;
    let output = jstr::strip(&result.stdout).to_string();
    if result.exit_code != 0 || jstr::is_blank(&output) {
        if !with_reason {
            return Err(RekallError::illegal(failure));
        }
        let error = jstr::strip(&result.stderr);
        let detail = if jstr::is_blank(error) { "there is no commit yet" } else { error };
        return Err(RekallError::illegal(format!("{failure}: {detail}")));
    }
    Ok(output)
}

/// The patch this commit introduced. Best effort: a diff that cannot be produced is dropped
/// rather than failing the commit that was otherwise read cleanly.
async fn diff_of(folder: &Path, hash: &str) -> Option<String> {
    match run(folder, &["show", "--format=", hash, "--"], None, READ_TIMEOUT).await {
        Ok(result) if result.ok() => Some(jstr::strip(&result.stdout).to_string()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------- inspector

/// What a project's folder is, git-wise. `branch` is absent on a detached head; the identity is
/// absent when git has none configured there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryStatus {
    pub folder: Option<String>,
    pub exists: bool,
    pub repository: bool,
    pub branch: Option<String>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
}

impl RepositoryStatus {
    pub fn unset() -> Self {
        Self { folder: None, exists: false, repository: false, branch: None, user_name: None, user_email: None }
    }

    pub fn missing(folder: &str) -> Self {
        Self { folder: Some(folder.to_string()), ..Self::unset() }
    }

    pub fn not_a_repository(folder: &str) -> Self {
        Self { folder: Some(folder.to_string()), exists: true, ..Self::unset() }
    }

    /// Whether git could commit here as someone: a repository with an email to sign with.
    pub fn can_commit(&self) -> bool {
        self.repository && self.user_email.is_some()
    }
}

/// Read-only questions about a folder: is it a repository, which branch is checked out, who
/// would git commit as (what `git config` resolves inside the folder).
#[derive(Clone, Copy, Debug, Default)]
pub struct GitRepositoryInspector;

impl GitRepositoryInspector {
    pub async fn is_repository(&self, folder: Option<&str>) -> bool {
        self.inspect(folder).await.repository
    }

    pub async fn inspect(&self, folder: Option<&str>) -> RepositoryStatus {
        let Some(folder) = folder.filter(|f| !jstr::is_blank(f)) else {
            return RepositoryStatus::unset();
        };
        let path = jstr::strip(folder);
        let repo_folder = PathBuf::from(path);
        if !repo_folder.is_dir() {
            return RepositoryStatus::missing(path);
        }
        let inside = match run(&repo_folder, &["rev-parse", "--is-inside-work-tree"], None, COMMAND_TIMEOUT).await {
            Ok(inside) => inside,
            Err(_) => return RepositoryStatus::not_a_repository(path),
        };
        if !inside.ok() || inside.output() != "true" {
            return RepositoryStatus::not_a_repository(path);
        }
        RepositoryStatus {
            folder: Some(path.to_string()),
            exists: true,
            repository: true,
            branch: answer(&repo_folder, &["symbolic-ref", "--short", "-q", "HEAD"]).await,
            user_name: answer(&repo_folder, &["config", "--get", "user.name"]).await,
            user_email: answer(&repo_folder, &["config", "--get", "user.email"]).await,
        }
    }
}

async fn answer(folder: &Path, arguments: &[&str]) -> Option<String> {
    match run(folder, arguments, None, COMMAND_TIMEOUT).await {
        Ok(result) if result.ok() && !jstr::is_blank(result.output()) => Some(result.output().to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_untracked_file_reads_as_added() {
        assert_eq!(PendingChange::parse("?? src/new.ts").unwrap(), PendingChange::new(PendingChangeKind::Added, "src/new.ts"));
    }

    #[test]
    fn a_staged_addition_and_a_tracked_modification_keep_their_kind() {
        assert_eq!(PendingChange::parse("A  src/staged.ts").unwrap().kind, PendingChangeKind::Added);
        assert_eq!(PendingChange::parse(" M src/edited.ts").unwrap().kind, PendingChangeKind::Modified);
        assert_eq!(PendingChange::parse("MM src/both.ts").unwrap().kind, PendingChangeKind::Modified);
    }

    #[test]
    fn a_deletion_staged_or_not_reads_as_deleted() {
        assert_eq!(PendingChange::parse(" D gone.ts").unwrap().kind, PendingChangeKind::Deleted);
        assert_eq!(PendingChange::parse("D  gone.ts").unwrap().kind, PendingChangeKind::Deleted);
    }

    #[test]
    fn a_rename_keeps_the_new_path() {
        assert_eq!(PendingChange::parse("R  old.ts -> new.ts").unwrap(), PendingChange::new(PendingChangeKind::Renamed, "new.ts"));
    }

    #[test]
    fn a_line_too_short_to_be_a_status_line_is_refused() {
        let error = PendingChange::parse("M ").unwrap_err();
        assert!(error.message().contains("Unexpected git status line"));
    }

    #[test]
    fn a_path_prints_the_way_java_printed_it() {
        assert_eq!(display_path(Path::new("/repo//vega/")), "/repo/vega");
        assert_eq!(display_path(Path::new("/")), "/");
    }
}
