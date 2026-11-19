use std::path::Path;

use rekall_kernel::{jstr, RekallError, Result};

use super::git_command::READ_TIMEOUT;
use super::{display_path, run};

/// How many untracked files are read into the patch; a folder full of build output is not a change.
const UNTRACKED_LIMIT: usize = 50;

/// Reads what a folder holds that no commit does yet. Never writes: an untracked file is shown as
/// added without being staged.
#[derive(Clone, Copy, Debug, Default)]
pub struct GitWorkingTreeReader;

impl GitWorkingTreeReader {
    /// The patch of every tracked change against `HEAD`, staged or not, followed by each untracked
    /// file as a new one. Empty when the tree is clean.
    pub async fn patch(&self, folder: &Path) -> Result<String> {
        let shown = display_path(folder);
        let tracked = run(folder, &["diff", "HEAD", "--no-color", "--no-ext-diff", "-M", "--"], None, READ_TIMEOUT).await?;
        if !tracked.ok() {
            return Err(RekallError::illegal(format!("Could not read the changes in {shown}: {}", jstr::strip(&tracked.stderr))));
        }
        let mut patch = tracked.stdout;
        for path in self.untracked(folder).await?.iter().take(UNTRACKED_LIMIT) {
            patch.push_str(&self.added_file(folder, path).await?);
        }
        Ok(patch)
    }

    async fn untracked(&self, folder: &Path) -> Result<Vec<String>> {
        let listed = run(folder, &["ls-files", "--others", "--exclude-standard", "-z"], None, READ_TIMEOUT).await?;
        if !listed.ok() {
            return Err(RekallError::illegal(format!(
                "Could not list the new files in {}: {}",
                display_path(folder),
                jstr::strip(&listed.stderr)
            )));
        }
        Ok(listed.stdout.split('\0').filter(|path| !path.is_empty()).map(str::to_string).collect())
    }

    /// `git diff --no-index` exits with 1 when the files differ, which is the case being read.
    async fn added_file(&self, folder: &Path, path: &str) -> Result<String> {
        let result = run(folder, &["diff", "--no-index", "--no-color", "--", "/dev/null", path], None, READ_TIMEOUT).await?;
        if result.exit_code > 1 {
            return Err(RekallError::illegal(format!("Could not read {path}: {}", jstr::strip(&result.stderr))));
        }
        Ok(result.stdout)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/commit/git/git_working_tree_reader_tests.rs"]
mod tests;
