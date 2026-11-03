use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use rekall_kernel::{jstr, Instant, RekallError, Result};

use super::{CommitTouch, LogEntry, LoggedCommit, display_path, run};
use super::git_command::READ_TIMEOUT;

/// ASCII unit separator: not going to show up in a commit subject by accident.
const FIELD_SEPARATOR: char = '\u{1F}';
/// ASCII record separator: opens each commit in the output of a log that lists files.
const RECORD_SEPARATOR: char = '\u{1E}';
const TOUCHES_TIMEOUT: Duration = Duration::from_secs(1);

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
        require_repository(folder)?;
        let tip = reference == "HEAD";
        if !tip {
            require_hash(reference)?;
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

    /// The commits after `since` up to `HEAD` that changed one of `paths` (literal names, files or
    /// folders), each with the files it changed there. Refused when git does not know `since`
    /// or takes longer than a second.
    pub async fn touching_since(&self, folder: &Path, since: &str, paths: &[String]) -> Result<Vec<CommitTouch>> {
        require_repository(folder)?;
        require_hash(since)?;
        let range = format!("{since}..HEAD");
        let format = format!("--format={RECORD_SEPARATOR}%H");
        let mut arguments = vec!["--literal-pathspecs", "log", "--name-only", format.as_str(), range.as_str(), "--"];
        arguments.extend(paths.iter().map(String::as_str));
        let result = run(folder, &arguments, None, TOUCHES_TIMEOUT).await?;
        if result.exit_code != 0 {
            return Err(RekallError::illegal(format!(
                "Could not read the log since {since} in {}: {}",
                display_path(folder),
                jstr::strip(&result.stderr)
            )));
        }
        Ok(result.stdout.split(RECORD_SEPARATOR).filter_map(touch_of).collect())
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

fn require_hash(reference: &str) -> Result<()> {
    static HASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-fA-F]{4,40}$").expect("a valid pattern"));
    if !HASH.is_match(reference) {
        return Err(RekallError::illegal(format!(
            "'{reference}' is not a commit hash: paste 4 to 40 hex characters from git log."
        )));
    }
    Ok(())
}

fn touch_of(record: &str) -> Option<CommitTouch> {
    let mut lines = record.lines().map(jstr::strip).filter(|line| !line.is_empty());
    let hash = lines.next()?.to_string();
    Some(CommitTouch { hash, files: lines.map(str::to_string).collect() })
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
